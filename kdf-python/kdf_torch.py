"""
KDF PyTorch Integration

Provides PyTorch nn.Module wrappers for KDF operations.

Usage:
    import torch
    from kdf_torch import KdfLayer, KdfLoss, KdfDataset

    # As a layer in a model
    class MyModel(nn.Module):
        def __init__(self):
            super().__init__()
            self.encoder = nn.Linear(768, 128)
            self.kdf = KdfLayer(threshold=0.85)

        def forward(self, x):
            embeddings = self.encoder(x)
            # KDF for inference (deduplication)
            selected, mask = self.kdf(embeddings)
            return selected

    # KDF-aware loss function
    loss_fn = KdfLoss(threshold=0.85, rare_weight=2.0)

    # KDF-based dataset sampling
    dataset = KdfDataset(data, threshold=0.85)

Requirements:
    pip install torch numpy
    maturin develop --release  (for kdf_rs)
"""

from typing import List, Optional, Tuple, Union, Callable
import numpy as np

# PyTorch imports
try:
    import torch
    import torch.nn as nn
    from torch.utils.data import Dataset, Sampler
    HAS_TORCH = True
except ImportError:
    HAS_TORCH = False
    # Fallback
    class nn:
        class Module:
            pass

# Try to import kdf_rs
try:
    from kdf_rs import Kdf, KdfResult, Layer
    HAS_KDF_RS = True
except ImportError:
    HAS_KDF_RS = False


class KdfLayer(nn.Module):
    """
    PyTorch layer for KDF-based sample selection.

    Applies KDF to select representative and rare samples from embeddings.
    Useful for deduplication, outlier detection, and efficient batch processing.

    Parameters
    ----------
    threshold : float, default=0.85
        Cosine similarity threshold.

    alpha_core : float, default=2.0
        Core layer decay rate.

    alpha_edge : float, default=1.5
        Edge layer decay rate.

    alpha_rare : float, default=0.3
        Rare layer decay rate.

    return_weights : bool, default=False
        If True, return weights for all samples instead of mask.

    differentiable : bool, default=False
        If True, use soft (differentiable) selection via sigmoid.
        If False, use hard selection (non-differentiable).

    Examples
    --------
    >>> kdf_layer = KdfLayer(threshold=0.85)
    >>> embeddings = torch.randn(100, 128)
    >>> selected, mask = kdf_layer(embeddings)
    >>> print(f"Selected {selected.shape[0]} out of {embeddings.shape[0]}")
    """

    def __init__(
        self,
        threshold: float = 0.85,
        alpha_core: float = 2.0,
        alpha_edge: float = 1.5,
        alpha_rare: float = 0.3,
        return_weights: bool = False,
        differentiable: bool = False,
    ):
        super().__init__()

        if not HAS_TORCH:
            raise ImportError("PyTorch is required. Install with: pip install torch")
        if not HAS_KDF_RS:
            raise ImportError(
                "kdf_rs is required. Build with: cd kdf-python && maturin develop --release"
            )

        self.threshold = threshold
        self.alpha_core = alpha_core
        self.alpha_edge = alpha_edge
        self.alpha_rare = alpha_rare
        self.return_weights = return_weights
        self.differentiable = differentiable

        # Store KDF processor
        self._kdf = Kdf(
            alpha_core=alpha_core,
            alpha_edge=alpha_edge,
            alpha_rare=alpha_rare,
        )

    def forward(
        self,
        x: torch.Tensor,
    ) -> Tuple[torch.Tensor, torch.Tensor]:
        """
        Forward pass: apply KDF selection.

        Parameters
        ----------
        x : torch.Tensor of shape (batch_size, embedding_dim)
            Input embeddings.

        Returns
        -------
        selected : torch.Tensor
            Selected embeddings (if not differentiable) or weighted embeddings.

        mask_or_weights : torch.Tensor
            Boolean mask of selected samples (if not differentiable)
            or soft weights (if differentiable).
        """
        # Normalize for cosine similarity
        x_normalized = torch.nn.functional.normalize(x, p=2, dim=1)

        # Convert to numpy for KDF processing
        x_np = x_normalized.detach().cpu().numpy()
        vectors = x_np.tolist()

        # Apply KDF
        result = self._kdf.process_vectors(vectors, threshold=self.threshold)

        if self.return_weights:
            # Return weights for all samples
            weights = torch.tensor(result.weights, dtype=x.dtype, device=x.device)
            weighted = x * weights.unsqueeze(1)
            return weighted, weights

        elif self.differentiable:
            # Soft selection using weights as sigmoid-like scores
            weights = torch.tensor(result.weights, dtype=x.dtype, device=x.device)
            # Scale weights to [0, 1] range more aggressively
            soft_mask = torch.sigmoid((weights - 0.5) * 10)
            weighted = x * soft_mask.unsqueeze(1)
            return weighted, soft_mask

        else:
            # Hard selection
            selected_indices = result.selected
            mask = torch.zeros(x.shape[0], dtype=torch.bool, device=x.device)
            mask[selected_indices] = True

            selected = x[mask]
            return selected, mask

    def get_layers(self, x: torch.Tensor) -> Tuple[torch.Tensor, torch.Tensor, torch.Tensor]:
        """
        Get layer classification for each sample.

        Returns
        -------
        core_mask, edge_mask, rare_mask : torch.Tensor
            Boolean masks for each layer.
        """
        x_normalized = torch.nn.functional.normalize(x, p=2, dim=1)
        x_np = x_normalized.detach().cpu().numpy()
        result = self._kdf.process_vectors(x_np.tolist(), threshold=self.threshold)

        n = x.shape[0]
        core_mask = torch.zeros(n, dtype=torch.bool, device=x.device)
        edge_mask = torch.zeros(n, dtype=torch.bool, device=x.device)
        rare_mask = torch.zeros(n, dtype=torch.bool, device=x.device)

        for i in result.core_items():
            core_mask[i] = True
        for i in result.edge_items():
            edge_mask[i] = True
        for i in result.rare_items():
            rare_mask[i] = True

        return core_mask, edge_mask, rare_mask

    def extra_repr(self) -> str:
        return f"threshold={self.threshold}, differentiable={self.differentiable}"


class KdfLoss(nn.Module):
    """
    KDF-aware loss function.

    Applies different weights to samples based on their KDF layer:
    - Rare samples: Higher weight (important for learning)
    - Core samples: Lower weight (redundant)
    - Edge samples: Normal weight

    Parameters
    ----------
    threshold : float, default=0.85
        Similarity threshold for KDF.

    base_loss : nn.Module, default=nn.CrossEntropyLoss(reduction='none')
        The base loss function (must have reduction='none').

    rare_weight : float, default=2.0
        Weight multiplier for Rare samples.

    edge_weight : float, default=1.0
        Weight multiplier for Edge samples.

    core_weight : float, default=0.5
        Weight multiplier for Core samples.

    Examples
    --------
    >>> loss_fn = KdfLoss(threshold=0.85, rare_weight=2.0)
    >>> embeddings = encoder(x)  # (batch, embed_dim)
    >>> logits = classifier(embeddings)  # (batch, num_classes)
    >>> loss = loss_fn(embeddings, logits, targets)
    """

    def __init__(
        self,
        threshold: float = 0.85,
        base_loss: Optional[nn.Module] = None,
        rare_weight: float = 2.0,
        edge_weight: float = 1.0,
        core_weight: float = 0.5,
    ):
        super().__init__()

        if not HAS_TORCH:
            raise ImportError("PyTorch is required")
        if not HAS_KDF_RS:
            raise ImportError("kdf_rs is required")

        self.threshold = threshold
        self.base_loss = base_loss or nn.CrossEntropyLoss(reduction='none')
        self.rare_weight = rare_weight
        self.edge_weight = edge_weight
        self.core_weight = core_weight

        self._kdf = Kdf()

    def forward(
        self,
        embeddings: torch.Tensor,
        predictions: torch.Tensor,
        targets: torch.Tensor,
    ) -> torch.Tensor:
        """
        Compute KDF-weighted loss.

        Parameters
        ----------
        embeddings : torch.Tensor of shape (batch_size, embedding_dim)
            Embeddings used for KDF layer classification.

        predictions : torch.Tensor
            Model predictions (logits).

        targets : torch.Tensor
            Ground truth labels.

        Returns
        -------
        loss : torch.Tensor
            Weighted loss (scalar).
        """
        # Compute base loss per sample
        per_sample_loss = self.base_loss(predictions, targets)

        # Get KDF weights
        x_normalized = torch.nn.functional.normalize(embeddings, p=2, dim=1)
        x_np = x_normalized.detach().cpu().numpy()
        result = self._kdf.process_vectors(x_np.tolist(), threshold=self.threshold)

        # Create weight tensor
        weights = torch.ones(embeddings.shape[0], dtype=embeddings.dtype, device=embeddings.device)

        for i in result.core_items():
            weights[i] = self.core_weight
        for i in result.edge_items():
            weights[i] = self.edge_weight
        for i in result.rare_items():
            weights[i] = self.rare_weight

        # Weighted loss
        weighted_loss = per_sample_loss * weights
        return weighted_loss.mean()


class KdfContrastiveLoss(nn.Module):
    """
    KDF-informed contrastive loss.

    Uses KDF layer information to select better negatives:
    - Edge samples as hard negatives
    - Core samples as easy negatives
    - Avoids Rare samples (potential noise)

    Parameters
    ----------
    threshold : float, default=0.85
        KDF similarity threshold.

    temperature : float, default=0.07
        Temperature for contrastive loss.

    use_hard_negatives : bool, default=True
        Prioritize Edge samples as hard negatives.

    Examples
    --------
    >>> loss_fn = KdfContrastiveLoss(threshold=0.85)
    >>> embeddings = encoder(x)
    >>> loss = loss_fn(embeddings, labels)
    """

    def __init__(
        self,
        threshold: float = 0.85,
        temperature: float = 0.07,
        use_hard_negatives: bool = True,
    ):
        super().__init__()

        if not HAS_TORCH:
            raise ImportError("PyTorch is required")
        if not HAS_KDF_RS:
            raise ImportError("kdf_rs is required")

        self.threshold = threshold
        self.temperature = temperature
        self.use_hard_negatives = use_hard_negatives
        self._kdf = Kdf()

    def forward(
        self,
        embeddings: torch.Tensor,
        labels: torch.Tensor,
    ) -> torch.Tensor:
        """
        Compute KDF-informed contrastive loss.

        Parameters
        ----------
        embeddings : torch.Tensor of shape (batch_size, embedding_dim)
            Normalized embeddings.

        labels : torch.Tensor of shape (batch_size,)
            Labels for positive pair identification.

        Returns
        -------
        loss : torch.Tensor
            Contrastive loss (scalar).
        """
        # Normalize embeddings
        embeddings = torch.nn.functional.normalize(embeddings, p=2, dim=1)

        # Get KDF layers
        x_np = embeddings.detach().cpu().numpy()
        result = self._kdf.process_vectors(x_np.tolist(), threshold=self.threshold)

        # Similarity matrix
        sim_matrix = torch.mm(embeddings, embeddings.t()) / self.temperature

        # Create masks
        batch_size = embeddings.shape[0]
        labels = labels.view(-1, 1)
        positive_mask = (labels == labels.t()).float()
        negative_mask = 1 - positive_mask

        # Remove self-similarity
        identity_mask = torch.eye(batch_size, device=embeddings.device)
        positive_mask = positive_mask - identity_mask

        # Apply KDF-based negative weighting
        if self.use_hard_negatives:
            neg_weights = torch.ones(batch_size, device=embeddings.device)
            # Edge = hard negatives (weight 2.0)
            for i in result.edge_items():
                neg_weights[i] = 2.0
            # Core = easy negatives (weight 0.5)
            for i in result.core_items():
                neg_weights[i] = 0.5
            # Rare = avoid (weight 0.1)
            for i in result.rare_items():
                neg_weights[i] = 0.1

            # Apply weights to negative mask
            negative_mask = negative_mask * neg_weights.unsqueeze(0)

        # Compute loss (InfoNCE style)
        exp_sim = torch.exp(sim_matrix)
        positive_sim = (exp_sim * positive_mask).sum(1)
        negative_sim = (exp_sim * negative_mask).sum(1)

        # Avoid division by zero
        loss = -torch.log(positive_sim / (positive_sim + negative_sim + 1e-8))

        return loss.mean()


class KdfSampler(Sampler):
    """
    PyTorch Sampler that uses KDF for intelligent batch sampling.

    Ensures each batch contains:
    - At least one Rare sample (if available)
    - Balanced representation of layers

    Parameters
    ----------
    embeddings : np.ndarray or torch.Tensor
        Precomputed embeddings for all samples.

    threshold : float, default=0.85
        KDF similarity threshold.

    batch_size : int, default=32
        Batch size.

    rare_per_batch : int, default=1
        Minimum Rare samples per batch.

    shuffle : bool, default=True
        Shuffle within layers.

    Examples
    --------
    >>> sampler = KdfSampler(embeddings, batch_size=32)
    >>> dataloader = DataLoader(dataset, batch_sampler=sampler)
    """

    def __init__(
        self,
        embeddings: Union[np.ndarray, 'torch.Tensor'],
        threshold: float = 0.85,
        batch_size: int = 32,
        rare_per_batch: int = 1,
        shuffle: bool = True,
    ):
        if not HAS_TORCH:
            raise ImportError("PyTorch is required")
        if not HAS_KDF_RS:
            raise ImportError("kdf_rs is required")

        # Convert to numpy
        if isinstance(embeddings, torch.Tensor):
            embeddings = embeddings.detach().cpu().numpy()

        self.embeddings = embeddings
        self.threshold = threshold
        self.batch_size = batch_size
        self.rare_per_batch = rare_per_batch
        self.shuffle = shuffle

        # Run KDF
        kdf = Kdf()
        embeddings_normalized = embeddings / (np.linalg.norm(embeddings, axis=1, keepdims=True) + 1e-10)
        result = kdf.process_vectors(embeddings_normalized.tolist(), threshold=threshold)

        self.core_indices = list(result.core_items())
        self.edge_indices = list(result.edge_items())
        self.rare_indices = list(result.rare_items())

        self._prepare_batches()

    def _prepare_batches(self):
        """Prepare batches with layer-balanced sampling."""
        if self.shuffle:
            np.random.shuffle(self.core_indices)
            np.random.shuffle(self.edge_indices)
            np.random.shuffle(self.rare_indices)

        self.batches = []
        rare_idx = 0
        edge_idx = 0
        core_idx = 0

        n_samples = len(self.embeddings)
        n_batches = (n_samples + self.batch_size - 1) // self.batch_size

        for _ in range(n_batches):
            batch = []

            # Add Rare samples
            for _ in range(min(self.rare_per_batch, len(self.rare_indices))):
                if rare_idx < len(self.rare_indices):
                    batch.append(self.rare_indices[rare_idx])
                    rare_idx = (rare_idx + 1) % max(1, len(self.rare_indices))

            # Fill remaining with Edge and Core
            remaining = self.batch_size - len(batch)
            edge_count = remaining // 2
            core_count = remaining - edge_count

            for _ in range(edge_count):
                if edge_idx < len(self.edge_indices):
                    batch.append(self.edge_indices[edge_idx])
                    edge_idx += 1
                elif core_idx < len(self.core_indices):
                    batch.append(self.core_indices[core_idx])
                    core_idx += 1

            for _ in range(core_count):
                if core_idx < len(self.core_indices):
                    batch.append(self.core_indices[core_idx])
                    core_idx += 1
                elif edge_idx < len(self.edge_indices):
                    batch.append(self.edge_indices[edge_idx])
                    edge_idx += 1

            if batch:
                self.batches.append(batch)

    def __iter__(self):
        if self.shuffle:
            self._prepare_batches()
        for batch in self.batches:
            yield batch

    def __len__(self):
        return len(self.batches)


class KdfDataset(Dataset):
    """
    PyTorch Dataset wrapper that applies KDF filtering.

    Filters a dataset to include only KDF-selected samples.

    Parameters
    ----------
    dataset : Dataset
        Original PyTorch dataset.

    embeddings : np.ndarray or torch.Tensor
        Precomputed embeddings.

    threshold : float, default=0.85
        KDF similarity threshold.

    include_layers : list, default=['rare', 'edge']
        Which layers to include. Options: 'core', 'edge', 'rare'.

    Examples
    --------
    >>> original_dataset = MyDataset(data)
    >>> embeddings = compute_embeddings(original_dataset)
    >>> kdf_dataset = KdfDataset(original_dataset, embeddings, threshold=0.85)
    >>> # kdf_dataset contains only selected samples
    """

    def __init__(
        self,
        dataset: Dataset,
        embeddings: Union[np.ndarray, 'torch.Tensor'],
        threshold: float = 0.85,
        include_layers: List[str] = None,
    ):
        if not HAS_TORCH:
            raise ImportError("PyTorch is required")
        if not HAS_KDF_RS:
            raise ImportError("kdf_rs is required")

        self.dataset = dataset
        self.threshold = threshold
        self.include_layers = include_layers or ['rare', 'edge']

        # Convert to numpy
        if isinstance(embeddings, torch.Tensor):
            embeddings = embeddings.detach().cpu().numpy()

        # Normalize
        embeddings = embeddings / (np.linalg.norm(embeddings, axis=1, keepdims=True) + 1e-10)

        # Run KDF
        kdf = Kdf()
        result = kdf.process_vectors(embeddings.tolist(), threshold=threshold)

        # Collect indices based on layers
        self.indices = []
        if 'core' in self.include_layers:
            self.indices.extend(result.core_items())
        if 'edge' in self.include_layers:
            self.indices.extend(result.edge_items())
        if 'rare' in self.include_layers:
            self.indices.extend(result.rare_items())

        self.indices = sorted(set(self.indices))

    def __getitem__(self, idx):
        original_idx = self.indices[idx]
        return self.dataset[original_idx]

    def __len__(self):
        return len(self.indices)


def demo():
    """Demo function showing PyTorch integration."""
    print("=== KDF PyTorch Integration Demo ===\n")

    if not HAS_TORCH:
        print("Please install PyTorch: pip install torch")
        return

    if not HAS_KDF_RS:
        print("Please build kdf_rs: cd kdf-python && maturin develop --release")
        return

    # Generate sample data
    torch.manual_seed(42)

    # Create embeddings: clusters + outliers
    cluster1 = torch.randn(30, 64) * 0.3 + torch.tensor([1.0] * 64)
    cluster2 = torch.randn(20, 64) * 0.3 + torch.tensor([-1.0] * 64)
    outliers = torch.randn(10, 64) * 1.5

    embeddings = torch.cat([cluster1, cluster2, outliers], dim=0)
    print(f"Input: {embeddings.shape[0]} samples, {embeddings.shape[1]} dimensions\n")

    # =========================================================================
    # 1. KdfLayer
    # =========================================================================
    print("--- KdfLayer ---")
    kdf_layer = KdfLayer(threshold=0.8)

    selected, mask = kdf_layer(embeddings)
    print(f"Selected: {selected.shape[0]} samples")
    print(f"Mask sum: {mask.sum().item()}")

    core_mask, edge_mask, rare_mask = kdf_layer.get_layers(embeddings)
    print(f"Core: {core_mask.sum().item()}, Edge: {edge_mask.sum().item()}, Rare: {rare_mask.sum().item()}")
    print()

    # =========================================================================
    # 2. KdfLayer with weights
    # =========================================================================
    print("--- KdfLayer (weighted) ---")
    kdf_weighted = KdfLayer(threshold=0.8, return_weights=True)
    weighted, weights = kdf_weighted(embeddings)
    print(f"Weight range: [{weights.min():.3f}, {weights.max():.3f}]")
    print(f"Mean weight: {weights.mean():.3f}")
    print()

    # =========================================================================
    # 3. KdfLoss
    # =========================================================================
    print("--- KdfLoss ---")
    loss_fn = KdfLoss(threshold=0.8, rare_weight=2.0, core_weight=0.5)

    # Simulate classification
    logits = torch.randn(60, 3)
    targets = torch.randint(0, 3, (60,))

    loss = loss_fn(embeddings, logits, targets)
    print(f"KDF-weighted loss: {loss.item():.4f}")
    print()

    # =========================================================================
    # 4. KdfSampler
    # =========================================================================
    print("--- KdfSampler ---")
    sampler = KdfSampler(embeddings, threshold=0.8, batch_size=16, rare_per_batch=2)

    print(f"Number of batches: {len(sampler)}")
    for i, batch in enumerate(sampler):
        if i < 2:
            print(f"Batch {i}: {len(batch)} samples, indices: {batch[:5]}...")
    print()

    # =========================================================================
    # 5. Model integration example
    # =========================================================================
    print("--- Model Integration Example ---")

    class SimpleEncoder(nn.Module):
        def __init__(self, input_dim, hidden_dim):
            super().__init__()
            self.encoder = nn.Sequential(
                nn.Linear(input_dim, hidden_dim),
                nn.ReLU(),
                nn.Linear(hidden_dim, hidden_dim),
            )
            self.kdf = KdfLayer(threshold=0.85)

        def forward(self, x, apply_kdf=True):
            embeddings = self.encoder(x)
            if apply_kdf:
                selected, mask = self.kdf(embeddings)
                return selected, mask
            return embeddings, None

    model = SimpleEncoder(64, 32)
    x = embeddings[:20]

    # Training mode (no KDF)
    out, _ = model(x, apply_kdf=False)
    print(f"Training output: {out.shape}")

    # Inference mode (with KDF)
    selected, mask = model(x, apply_kdf=True)
    print(f"Inference output: {selected.shape} (from {x.shape[0]} inputs)")
    print()

    print("=== Demo Complete ===")


if __name__ == "__main__":
    demo()
