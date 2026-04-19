"""
KDF Embeddings Integration

Provides sentence-transformers integration for KDF text processing.

Usage:
    from kdf_embeddings import KdfEmbeddings

    # Initialize with a model (default: paraphrase-MiniLM-L6-v2)
    kdf = KdfEmbeddings()

    # Process texts - automatically embeds and applies KDF
    texts = ["Hello world", "Hello there", "Goodbye world", "Random text"]
    result = kdf.process(texts, threshold=0.8)

    # Access results
    print(f"Selected: {result.selected}")
    print(f"Rare items: {result.rare_items()}")

    # Get selected texts directly
    selected_texts = kdf.get_selected_texts(texts, result)
    print(f"Selected texts: {selected_texts}")

Requirements:
    pip install sentence-transformers
    maturin develop --release  (for kdf_rs)
"""

from typing import List, Optional, Union, Callable
import numpy as np

# Try to import kdf_rs (built with maturin)
try:
    from kdf_rs import Kdf, KdfResult, Layer, AutoThresholdResult
    HAS_KDF_RS = True
except ImportError:
    HAS_KDF_RS = False
    print("Warning: kdf_rs not found. Build with: cd kdf-python && maturin develop --release")

# Try to import sentence-transformers
try:
    from sentence_transformers import SentenceTransformer
    HAS_SENTENCE_TRANSFORMERS = True
except ImportError:
    HAS_SENTENCE_TRANSFORMERS = False


class KdfEmbeddings:
    """
    KDF with sentence-transformers embeddings.

    Automatically converts text to embeddings and applies KDF for
    redundancy reduction while preserving rare items.

    Attributes:
        model_name: The sentence-transformers model to use
        model: The loaded SentenceTransformer model
        kdf: The KDF processor instance
        embeddings_cache: Optional cache for embeddings
    """

    # Popular models and their characteristics
    MODELS = {
        # Fast and lightweight
        "paraphrase-MiniLM-L6-v2": "Fast, good quality (default)",
        "all-MiniLM-L6-v2": "Fast, good for semantic search",

        # Higher quality
        "paraphrase-mpnet-base-v2": "Best quality, slower",
        "all-mpnet-base-v2": "High quality semantic search",

        # Multilingual
        "paraphrase-multilingual-MiniLM-L12-v2": "50+ languages",
        "distiluse-base-multilingual-cased-v2": "Multilingual, lightweight",
    }

    def __init__(
        self,
        model_name: str = "paraphrase-MiniLM-L6-v2",
        device: Optional[str] = None,
        cache_embeddings: bool = True,
        alpha_core: float = 2.0,
        alpha_edge: float = 1.5,
        alpha_rare: float = 0.3,
    ):
        """
        Initialize KDF with embeddings support.

        Args:
            model_name: sentence-transformers model name
            device: 'cpu', 'cuda', or None (auto-detect)
            cache_embeddings: Whether to cache computed embeddings
            alpha_core: KDF Core layer decay rate
            alpha_edge: KDF Edge layer decay rate
            alpha_rare: KDF Rare layer decay rate
        """
        if not HAS_SENTENCE_TRANSFORMERS:
            raise ImportError(
                "sentence-transformers is required. Install with: pip install sentence-transformers"
            )

        if not HAS_KDF_RS:
            raise ImportError(
                "kdf_rs is required. Build with: cd kdf-python && maturin develop --release"
            )

        self.model_name = model_name
        self.model = SentenceTransformer(model_name, device=device)
        self.kdf = Kdf(
            alpha_core=alpha_core,
            alpha_edge=alpha_edge,
            alpha_rare=alpha_rare,
        )
        self.cache_embeddings = cache_embeddings
        self._embeddings_cache: dict = {}

    def encode(
        self,
        texts: List[str],
        batch_size: int = 32,
        show_progress: bool = False,
        normalize: bool = True,
    ) -> np.ndarray:
        """
        Encode texts to embeddings.

        Args:
            texts: List of texts to encode
            batch_size: Batch size for encoding
            show_progress: Show progress bar
            normalize: Normalize embeddings to unit length

        Returns:
            numpy array of embeddings (n_texts, embedding_dim)
        """
        # Check cache
        if self.cache_embeddings:
            uncached_indices = []
            uncached_texts = []
            for i, text in enumerate(texts):
                if text not in self._embeddings_cache:
                    uncached_indices.append(i)
                    uncached_texts.append(text)

            if uncached_texts:
                new_embeddings = self.model.encode(
                    uncached_texts,
                    batch_size=batch_size,
                    show_progress_bar=show_progress,
                    normalize_embeddings=normalize,
                )
                for i, text in zip(uncached_indices, uncached_texts):
                    self._embeddings_cache[text] = new_embeddings[uncached_indices.index(i)]

            # Collect all embeddings
            embeddings = np.array([self._embeddings_cache[text] for text in texts])
        else:
            embeddings = self.model.encode(
                texts,
                batch_size=batch_size,
                show_progress_bar=show_progress,
                normalize_embeddings=normalize,
            )

        return embeddings

    def process(
        self,
        texts: List[str],
        threshold: float = 0.8,
        batch_size: int = 32,
        show_progress: bool = False,
    ) -> KdfResult:
        """
        Process texts with KDF using embeddings.

        Args:
            texts: List of texts to process
            threshold: Cosine similarity threshold (0.0-1.0)
            batch_size: Batch size for embedding
            show_progress: Show embedding progress

        Returns:
            KdfResult with selected items and layer classification
        """
        if not texts:
            raise ValueError("texts cannot be empty")

        # Encode texts to embeddings
        embeddings = self.encode(
            texts,
            batch_size=batch_size,
            show_progress=show_progress,
            normalize=True,  # Cosine similarity with normalized = dot product
        )

        # Convert to list of lists for kdf_rs
        vectors = embeddings.tolist()

        # Process with KDF
        result = self.kdf.process_vectors(vectors, threshold=threshold)

        return result

    def process_with_embeddings(
        self,
        embeddings: np.ndarray,
        threshold: float = 0.8,
    ) -> KdfResult:
        """
        Process pre-computed embeddings with KDF.

        Args:
            embeddings: Pre-computed embeddings (n_samples, embedding_dim)
            threshold: Cosine similarity threshold

        Returns:
            KdfResult
        """
        vectors = embeddings.tolist()
        return self.kdf.process_vectors(vectors, threshold=threshold)

    def process_auto(
        self,
        texts: List[str],
        batch_size: int = 32,
        show_progress: bool = False,
    ) -> AutoThresholdResult:
        """
        Process texts with automatic threshold selection.

        Automatically finds the optimal similarity threshold for the data.

        Args:
            texts: List of texts to process
            batch_size: Batch size for embedding
            show_progress: Show embedding progress

        Returns:
            AutoThresholdResult with optimal threshold and results
        """
        if not texts:
            raise ValueError("texts cannot be empty")

        # Encode texts to embeddings
        embeddings = self.encode(
            texts,
            batch_size=batch_size,
            show_progress=show_progress,
            normalize=True,
        )

        # Convert to list of lists for kdf_rs
        vectors = embeddings.tolist()

        # Process with auto-threshold
        return self.kdf.process_vectors_auto(vectors)

    def get_selected_texts(
        self,
        texts: List[str],
        result: KdfResult,
    ) -> List[str]:
        """
        Get the selected texts from a KDF result.

        Args:
            texts: Original text list
            result: KDF result

        Returns:
            List of selected texts
        """
        return [texts[i] for i in result.selected]

    def get_rare_texts(
        self,
        texts: List[str],
        result: KdfResult,
    ) -> List[str]:
        """
        Get the rare (isolated) texts from a KDF result.

        Args:
            texts: Original text list
            result: KDF result

        Returns:
            List of rare texts
        """
        return [texts[i] for i in result.rare_items()]

    def deduplicate(
        self,
        texts: List[str],
        threshold: float = 0.9,
    ) -> List[str]:
        """
        Remove duplicate texts while keeping rare ones.

        Convenience method for text deduplication.

        Args:
            texts: List of texts
            threshold: Similarity threshold for duplicates

        Returns:
            Deduplicated list of texts
        """
        result = self.process(texts, threshold=threshold)
        return self.get_selected_texts(texts, result)

    def find_outliers(
        self,
        texts: List[str],
        threshold: float = 0.7,
    ) -> List[str]:
        """
        Find outlier/anomalous texts.

        Args:
            texts: List of texts
            threshold: Similarity threshold

        Returns:
            List of outlier texts (Rare layer)
        """
        result = self.process(texts, threshold=threshold)
        return self.get_rare_texts(texts, result)

    def clear_cache(self):
        """Clear the embeddings cache."""
        self._embeddings_cache.clear()

    @classmethod
    def list_models(cls) -> dict:
        """List recommended models with descriptions."""
        return cls.MODELS.copy()


def demo():
    """Demo function showing KdfEmbeddings usage."""
    print("=== KDF Embeddings Demo ===\n")

    if not HAS_SENTENCE_TRANSFORMERS:
        print("Please install sentence-transformers: pip install sentence-transformers")
        return

    if not HAS_KDF_RS:
        print("Please build kdf_rs: cd kdf-python && maturin develop --release")
        return

    # Initialize
    print("Loading model...")
    kdf = KdfEmbeddings(model_name="paraphrase-MiniLM-L6-v2")

    # Sample texts
    texts = [
        # Cluster 1: Greetings
        "Hello, how are you?",
        "Hi there, how are you doing?",
        "Hey, what's up?",

        # Cluster 2: Weather
        "The weather is nice today",
        "It's a beautiful sunny day",
        "Nice weather we're having",

        # Cluster 3: Technical
        "Python is a programming language",
        "Python is great for data science",

        # Rare/Outliers
        "The quick brown fox jumps over the lazy dog",  # Classic pangram
        "42 is the answer to everything",  # Reference
    ]

    print(f"\nProcessing {len(texts)} texts...\n")

    # Process
    result = kdf.process(texts, threshold=0.75)

    # Show results
    print("--- Results ---")
    print(f"Total texts: {len(texts)}")
    print(f"Selected: {len(result.selected)}")
    print(f"Compression: {result.compression_rate()*100:.1f}%")
    print()

    print("Selected texts:")
    for idx in result.selected:
        layer = result.layers[idx]
        print(f"  [{idx}] ({layer}): {texts[idx][:50]}...")
    print()

    print("Rare (outlier) texts:")
    for idx in result.rare_items():
        print(f"  [{idx}]: {texts[idx]}")
    print()

    # Deduplication example
    print("--- Deduplication Example ---")
    duplicates = [
        "Machine learning is fascinating",
        "Machine learning is really fascinating",
        "I love machine learning",
        "Deep learning is a subset of ML",
        "Random unrelated text about cooking",
    ]
    deduped = kdf.deduplicate(duplicates, threshold=0.85)
    print(f"Original: {len(duplicates)} texts")
    print(f"After dedup: {len(deduped)} texts")
    for text in deduped:
        print(f"  - {text}")


if __name__ == "__main__":
    demo()
