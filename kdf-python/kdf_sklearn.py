"""
KDF Sklearn-Compatible API

Provides scikit-learn compatible transformers for KDF.

Usage:
    from kdf_sklearn import KdfTransformer, KdfVectorizer

    # For pre-computed vectors
    kdf = KdfTransformer(threshold=0.85)
    X_reduced = kdf.fit_transform(X)

    # For text data (with TF-IDF or embeddings)
    kdf = KdfVectorizer(threshold=0.85, use_embeddings=True)
    X_reduced = kdf.fit_transform(texts)

    # In a pipeline
    from sklearn.pipeline import Pipeline
    pipe = Pipeline([
        ('kdf', KdfTransformer(threshold=0.85)),
        ('clf', LogisticRegression())
    ])

Compatible with:
    - sklearn.pipeline.Pipeline
    - sklearn.model_selection.GridSearchCV
    - sklearn.base.clone()
    - sklearn transformers API (fit, transform, fit_transform)

Requirements:
    pip install scikit-learn numpy
    maturin develop --release  (for kdf_rs)
"""

from typing import List, Optional, Union, Any
import numpy as np

# sklearn imports
try:
    from sklearn.base import BaseEstimator, TransformerMixin
    from sklearn.utils.validation import check_is_fitted, check_array
    HAS_SKLEARN = True
except ImportError:
    HAS_SKLEARN = False
    # Fallback base classes for when sklearn is not available
    class BaseEstimator:
        pass
    class TransformerMixin:
        pass

# Try to import kdf_rs
try:
    from kdf_rs import Kdf, KdfResult, Layer
    HAS_KDF_RS = True
except ImportError:
    HAS_KDF_RS = False

# Try to import sentence-transformers
try:
    from sentence_transformers import SentenceTransformer
    HAS_SENTENCE_TRANSFORMERS = True
except ImportError:
    HAS_SENTENCE_TRANSFORMERS = False


class KdfTransformer(BaseEstimator, TransformerMixin):
    """
    Sklearn-compatible KDF transformer for vector data.

    Reduces redundancy in vector datasets while preserving rare/unique samples.
    Compatible with sklearn Pipeline, GridSearchCV, and other sklearn utilities.

    Parameters
    ----------
    threshold : float, default=0.85
        Cosine similarity threshold for grouping similar items.
        Higher values = more items retained (less aggressive deduplication).

    alpha_core : float, default=2.0
        Decay rate for Core layer (redundant items).

    alpha_edge : float, default=1.5
        Decay rate for Edge layer (partially redundant items).

    alpha_rare : float, default=0.3
        Decay rate for Rare layer (unique/isolated items).

    return_selected_only : bool, default=True
        If True, transform returns only selected samples.
        If False, returns all samples with weights applied.

    Attributes
    ----------
    result_ : KdfResult
        The KDF result from the last fit.

    selected_indices_ : ndarray of shape (n_selected,)
        Indices of selected samples.

    layers_ : list
        Layer classification for each sample.

    n_features_in_ : int
        Number of features seen during fit.

    Examples
    --------
    >>> from kdf_sklearn import KdfTransformer
    >>> import numpy as np
    >>> X = np.random.randn(100, 10)
    >>> kdf = KdfTransformer(threshold=0.85)
    >>> X_reduced = kdf.fit_transform(X)
    >>> print(f"Reduced from {X.shape[0]} to {X_reduced.shape[0]} samples")
    """

    def __init__(
        self,
        threshold: float = 0.85,
        alpha_core: float = 2.0,
        alpha_edge: float = 1.5,
        alpha_rare: float = 0.3,
        return_selected_only: bool = True,
    ):
        self.threshold = threshold
        self.alpha_core = alpha_core
        self.alpha_edge = alpha_edge
        self.alpha_rare = alpha_rare
        self.return_selected_only = return_selected_only

    def _check_dependencies(self):
        """Check required dependencies."""
        if not HAS_KDF_RS:
            raise ImportError(
                "kdf_rs is required. Build with: cd kdf-python && maturin develop --release"
            )
        if not HAS_SKLEARN:
            raise ImportError(
                "scikit-learn is required. Install with: pip install scikit-learn"
            )

    def fit(self, X, y=None):
        """
        Fit the KDF transformer.

        Parameters
        ----------
        X : array-like of shape (n_samples, n_features)
            Training data.

        y : None
            Ignored. Present for sklearn API compatibility.

        Returns
        -------
        self : KdfTransformer
            Fitted transformer.
        """
        self._check_dependencies()

        # Validate input
        X = check_array(X, accept_sparse=False, dtype=np.float64)
        self.n_features_in_ = X.shape[1]

        # Create KDF processor
        kdf = Kdf(
            alpha_core=self.alpha_core,
            alpha_edge=self.alpha_edge,
            alpha_rare=self.alpha_rare,
        )

        # Normalize for cosine similarity
        X_normalized = X / (np.linalg.norm(X, axis=1, keepdims=True) + 1e-10)

        # Process with KDF
        vectors = X_normalized.tolist()
        self.result_ = kdf.process_vectors(vectors, threshold=self.threshold)

        # Store results
        self.selected_indices_ = np.array(self.result_.selected)
        self.layers_ = self.result_.layers
        self.weights_ = np.array(self.result_.weights)

        return self

    def transform(self, X):
        """
        Transform data using fitted KDF.

        Parameters
        ----------
        X : array-like of shape (n_samples, n_features)
            Data to transform.

        Returns
        -------
        X_new : ndarray of shape (n_selected, n_features) or (n_samples, n_features)
            Transformed data. If return_selected_only=True, returns only selected samples.
            Otherwise, returns all samples with weights applied.
        """
        check_is_fitted(self, ['result_', 'selected_indices_'])
        X = check_array(X, accept_sparse=False, dtype=np.float64)

        if self.return_selected_only:
            # Return only selected samples
            return X[self.selected_indices_]
        else:
            # Return weighted samples
            return X * self.weights_.reshape(-1, 1)

    def fit_transform(self, X, y=None):
        """
        Fit and transform in one step.

        Parameters
        ----------
        X : array-like of shape (n_samples, n_features)
            Training data.

        y : None
            Ignored.

        Returns
        -------
        X_new : ndarray
            Transformed data.
        """
        self.fit(X, y)
        return self.transform(X)

    def get_support(self, indices: bool = True):
        """
        Get a mask or indices of selected samples.

        Parameters
        ----------
        indices : bool, default=True
            If True, return indices. If False, return boolean mask.

        Returns
        -------
        support : ndarray
            Selected sample indices or boolean mask.
        """
        check_is_fitted(self, ['selected_indices_', 'result_'])

        if indices:
            return self.selected_indices_
        else:
            mask = np.zeros(len(self.layers_), dtype=bool)
            mask[self.selected_indices_] = True
            return mask

    def get_layer_indices(self, layer: str = 'rare'):
        """
        Get indices of samples in a specific layer.

        Parameters
        ----------
        layer : str, default='rare'
            Layer name: 'core', 'edge', or 'rare'.

        Returns
        -------
        indices : ndarray
            Indices of samples in the specified layer.
        """
        check_is_fitted(self, ['result_'])

        layer_map = {
            'core': self.result_.core_items,
            'edge': self.result_.edge_items,
            'rare': self.result_.rare_items,
        }

        if layer.lower() not in layer_map:
            raise ValueError(f"Unknown layer: {layer}. Must be 'core', 'edge', or 'rare'.")

        return np.array(layer_map[layer.lower()]())

    def inverse_transform(self, X):
        """
        Inverse transform is not supported for KDF.

        KDF is a lossy transformation (samples are discarded).
        """
        raise NotImplementedError(
            "inverse_transform is not supported for KDF. "
            "KDF is a lossy transformation where samples are discarded."
        )

    def _more_tags(self):
        """sklearn tags for compatibility."""
        return {
            'requires_y': False,
            'preserves_dtype': [np.float64, np.float32],
            'stateless': False,
        }


class KdfVectorizer(BaseEstimator, TransformerMixin):
    """
    Sklearn-compatible KDF vectorizer for text data.

    Combines text vectorization with KDF for text deduplication.
    Supports TF-IDF or sentence-transformers embeddings.

    Parameters
    ----------
    threshold : float, default=0.85
        Similarity threshold for grouping.

    use_embeddings : bool, default=False
        If True, use sentence-transformers for embeddings.
        If False, use TF-IDF vectorization.

    embedding_model : str, default="paraphrase-MiniLM-L6-v2"
        Sentence-transformers model name (if use_embeddings=True).

    max_features : int, default=10000
        Maximum TF-IDF features (if use_embeddings=False).

    alpha_core : float, default=2.0
        Decay rate for Core layer.

    alpha_edge : float, default=1.5
        Decay rate for Edge layer.

    alpha_rare : float, default=0.3
        Decay rate for Rare layer.

    Attributes
    ----------
    vectorizer_ : TfidfVectorizer or SentenceTransformer
        The underlying vectorizer.

    kdf_transformer_ : KdfTransformer
        The KDF transformer.

    Examples
    --------
    >>> from kdf_sklearn import KdfVectorizer
    >>> texts = ["Hello world", "Hello there", "Random unique text"]
    >>> kdf = KdfVectorizer(threshold=0.8, use_embeddings=True)
    >>> X_reduced = kdf.fit_transform(texts)
    """

    def __init__(
        self,
        threshold: float = 0.85,
        use_embeddings: bool = False,
        embedding_model: str = "paraphrase-MiniLM-L6-v2",
        max_features: int = 10000,
        alpha_core: float = 2.0,
        alpha_edge: float = 1.5,
        alpha_rare: float = 0.3,
    ):
        self.threshold = threshold
        self.use_embeddings = use_embeddings
        self.embedding_model = embedding_model
        self.max_features = max_features
        self.alpha_core = alpha_core
        self.alpha_edge = alpha_edge
        self.alpha_rare = alpha_rare

    def _check_dependencies(self):
        """Check required dependencies."""
        if not HAS_KDF_RS:
            raise ImportError(
                "kdf_rs is required. Build with: cd kdf-python && maturin develop --release"
            )
        if not HAS_SKLEARN:
            raise ImportError(
                "scikit-learn is required. Install with: pip install scikit-learn"
            )
        if self.use_embeddings and not HAS_SENTENCE_TRANSFORMERS:
            raise ImportError(
                "sentence-transformers is required for embeddings. "
                "Install with: pip install sentence-transformers"
            )

    def fit(self, X, y=None):
        """
        Fit the KDF vectorizer.

        Parameters
        ----------
        X : array-like of shape (n_samples,)
            Text documents.

        y : None
            Ignored.

        Returns
        -------
        self : KdfVectorizer
        """
        self._check_dependencies()

        # Vectorize
        if self.use_embeddings:
            self.vectorizer_ = SentenceTransformer(self.embedding_model)
            vectors = self.vectorizer_.encode(
                list(X),
                normalize_embeddings=True,
                show_progress_bar=False,
            )
        else:
            from sklearn.feature_extraction.text import TfidfVectorizer
            self.vectorizer_ = TfidfVectorizer(max_features=self.max_features)
            vectors = self.vectorizer_.fit_transform(X).toarray()
            # Normalize
            vectors = vectors / (np.linalg.norm(vectors, axis=1, keepdims=True) + 1e-10)

        # Fit KDF
        self.kdf_transformer_ = KdfTransformer(
            threshold=self.threshold,
            alpha_core=self.alpha_core,
            alpha_edge=self.alpha_edge,
            alpha_rare=self.alpha_rare,
        )
        self.kdf_transformer_.fit(vectors)

        # Store for later
        self._fitted_vectors = vectors

        return self

    def transform(self, X):
        """
        Transform text data.

        Parameters
        ----------
        X : array-like of shape (n_samples,)
            Text documents.

        Returns
        -------
        X_new : ndarray
            Transformed vectors for selected samples.
        """
        check_is_fitted(self, ['vectorizer_', 'kdf_transformer_'])

        if self.use_embeddings:
            vectors = self.vectorizer_.encode(
                list(X),
                normalize_embeddings=True,
                show_progress_bar=False,
            )
        else:
            vectors = self.vectorizer_.transform(X).toarray()
            vectors = vectors / (np.linalg.norm(vectors, axis=1, keepdims=True) + 1e-10)

        return self.kdf_transformer_.transform(vectors)

    def fit_transform(self, X, y=None):
        """
        Fit and transform in one step.
        """
        self.fit(X, y)
        return self._fitted_vectors[self.kdf_transformer_.selected_indices_]

    def get_selected_texts(self, X) -> List[str]:
        """
        Get the selected texts after KDF processing.

        Parameters
        ----------
        X : array-like of shape (n_samples,)
            Original text documents.

        Returns
        -------
        selected_texts : list
            Texts that were selected by KDF.
        """
        check_is_fitted(self, ['kdf_transformer_'])
        texts = list(X)
        return [texts[i] for i in self.kdf_transformer_.selected_indices_]

    def get_rare_texts(self, X) -> List[str]:
        """
        Get texts classified as Rare (unique/outliers).

        Parameters
        ----------
        X : array-like of shape (n_samples,)
            Original text documents.

        Returns
        -------
        rare_texts : list
            Texts classified as Rare.
        """
        check_is_fitted(self, ['kdf_transformer_'])
        texts = list(X)
        rare_indices = self.kdf_transformer_.get_layer_indices('rare')
        return [texts[i] for i in rare_indices]


class KdfSampler(BaseEstimator, TransformerMixin):
    """
    Sklearn-compatible KDF sampler for imbalanced data.

    Uses KDF to intelligently sample data:
    - Preserves all Rare samples (potential anomalies or minority class)
    - Reduces Core samples (redundant majority class)
    - Balances Edge samples

    Useful for imbalanced classification tasks.

    Parameters
    ----------
    threshold : float, default=0.85
        Similarity threshold.

    preserve_ratio : float, default=1.0
        Ratio of Rare samples to preserve (1.0 = all).

    core_sample_ratio : float, default=0.1
        Ratio of Core samples to keep as representatives.

    Attributes
    ----------
    sample_indices_ : ndarray
        Indices of sampled data points.

    Examples
    --------
    >>> from kdf_sklearn import KdfSampler
    >>> X, y = make_imbalanced_dataset()
    >>> sampler = KdfSampler(threshold=0.8, core_sample_ratio=0.2)
    >>> X_sampled, y_sampled = sampler.fit_resample(X, y)
    """

    def __init__(
        self,
        threshold: float = 0.85,
        preserve_ratio: float = 1.0,
        core_sample_ratio: float = 0.1,
    ):
        self.threshold = threshold
        self.preserve_ratio = preserve_ratio
        self.core_sample_ratio = core_sample_ratio

    def fit(self, X, y=None):
        """Fit the sampler."""
        if not HAS_KDF_RS:
            raise ImportError("kdf_rs is required")
        if not HAS_SKLEARN:
            raise ImportError("scikit-learn is required")

        X = check_array(X)

        # Run KDF
        kdf = Kdf()
        X_normalized = X / (np.linalg.norm(X, axis=1, keepdims=True) + 1e-10)
        self.result_ = kdf.process_vectors(X_normalized.tolist(), threshold=self.threshold)

        # Compute sample indices
        sample_indices = []

        # All Rare samples (preserve_ratio)
        rare_indices = list(self.result_.rare_items())
        n_rare_keep = int(len(rare_indices) * self.preserve_ratio)
        sample_indices.extend(rare_indices[:n_rare_keep])

        # All Edge samples
        sample_indices.extend(self.result_.edge_items())

        # Sampled Core (representatives)
        core_indices = list(self.result_.core_items())
        n_core_keep = max(1, int(len(core_indices) * self.core_sample_ratio))
        # Keep evenly spaced cores
        if core_indices:
            step = max(1, len(core_indices) // n_core_keep)
            sample_indices.extend(core_indices[::step][:n_core_keep])

        self.sample_indices_ = np.array(sorted(set(sample_indices)))

        return self

    def transform(self, X):
        """Transform (sample) the data."""
        check_is_fitted(self, ['sample_indices_'])
        X = check_array(X)
        return X[self.sample_indices_]

    def fit_transform(self, X, y=None):
        """Fit and transform."""
        self.fit(X, y)
        return self.transform(X)

    def fit_resample(self, X, y):
        """
        Fit and resample (for compatibility with imbalanced-learn API).

        Parameters
        ----------
        X : array-like of shape (n_samples, n_features)
            Feature matrix.

        y : array-like of shape (n_samples,)
            Target labels.

        Returns
        -------
        X_resampled : ndarray
            Resampled feature matrix.

        y_resampled : ndarray
            Resampled target labels.
        """
        self.fit(X, y)
        X_resampled = X[self.sample_indices_]
        y_resampled = np.asarray(y)[self.sample_indices_]
        return X_resampled, y_resampled


def demo():
    """Demo function showing sklearn API usage."""
    print("=== KDF Sklearn API Demo ===\n")

    if not HAS_SKLEARN:
        print("Please install scikit-learn: pip install scikit-learn")
        return

    if not HAS_KDF_RS:
        print("Please build kdf_rs: cd kdf-python && maturin develop --release")
        return

    # Generate sample data
    np.random.seed(42)

    # Create clusters + outliers
    n_samples = 100
    n_features = 10

    # Cluster 1 (majority)
    cluster1 = np.random.randn(50, n_features) * 0.5 + np.array([1] * n_features)

    # Cluster 2
    cluster2 = np.random.randn(30, n_features) * 0.5 + np.array([-1] * n_features)

    # Outliers (rare)
    outliers = np.random.randn(20, n_features) * 2

    X = np.vstack([cluster1, cluster2, outliers])

    print(f"Original data: {X.shape[0]} samples, {X.shape[1]} features\n")

    # =========================================================================
    # 1. Basic KdfTransformer
    # =========================================================================
    print("--- KdfTransformer ---")
    kdf = KdfTransformer(threshold=0.8)
    X_reduced = kdf.fit_transform(X)

    print(f"Reduced to: {X_reduced.shape[0]} samples")
    print(f"Compression: {(1 - X_reduced.shape[0]/X.shape[0])*100:.1f}%")
    print(f"Rare samples: {len(kdf.get_layer_indices('rare'))}")
    print()

    # =========================================================================
    # 2. Pipeline integration
    # =========================================================================
    print("--- Pipeline Integration ---")
    from sklearn.pipeline import Pipeline
    from sklearn.preprocessing import StandardScaler

    pipe = Pipeline([
        ('scaler', StandardScaler()),
        ('kdf', KdfTransformer(threshold=0.85)),
    ])

    X_pipe = pipe.fit_transform(X)
    print(f"Pipeline output: {X_pipe.shape[0]} samples")
    print()

    # =========================================================================
    # 3. KdfSampler
    # =========================================================================
    print("--- KdfSampler ---")
    y = np.array([0]*50 + [1]*30 + [2]*20)  # Labels

    sampler = KdfSampler(threshold=0.8, core_sample_ratio=0.2)
    X_sampled, y_sampled = sampler.fit_resample(X, y)

    print(f"Resampled: {X_sampled.shape[0]} samples")
    print(f"Label distribution: {dict(zip(*np.unique(y_sampled, return_counts=True)))}")
    print()

    # =========================================================================
    # 4. get_support (feature selector style)
    # =========================================================================
    print("--- get_support ---")
    kdf2 = KdfTransformer(threshold=0.85)
    kdf2.fit(X)

    indices = kdf2.get_support(indices=True)
    mask = kdf2.get_support(indices=False)

    print(f"Selected indices: {indices[:10]}...")
    print(f"Mask sum: {mask.sum()} selected out of {len(mask)}")
    print()

    print("=== Demo Complete ===")


if __name__ == "__main__":
    demo()
