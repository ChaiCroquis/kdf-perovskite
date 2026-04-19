"""
KDF Visualization Module

Interactive visualization for KDF results in Jupyter notebooks.

Usage:
    from kdf_viz import KdfVisualizer

    viz = KdfVisualizer()
    viz.plot_layers(result, items)
    viz.plot_threshold_analysis(kdf, items)
    viz.interactive_threshold(kdf, items)

Requirements:
    pip install matplotlib plotly ipywidgets
"""

from typing import List, Optional, Any, Dict, Tuple
import warnings

# Optional imports with fallbacks
try:
    import matplotlib.pyplot as plt
    import matplotlib.patches as mpatches
    HAS_MATPLOTLIB = True
except ImportError:
    HAS_MATPLOTLIB = False

try:
    import plotly.graph_objects as go
    import plotly.express as px
    from plotly.subplots import make_subplots
    HAS_PLOTLY = True
except ImportError:
    HAS_PLOTLY = False

try:
    import ipywidgets as widgets
    from IPython.display import display, HTML
    HAS_WIDGETS = True
except ImportError:
    HAS_WIDGETS = False

try:
    import numpy as np
    HAS_NUMPY = True
except ImportError:
    HAS_NUMPY = False

try:
    from kdf_rs import Kdf, KdfResult, Layer
    HAS_KDF_RS = True
except ImportError:
    HAS_KDF_RS = False


# Layer colors
LAYER_COLORS = {
    "Core": "#3498db",   # Blue
    "Edge": "#f39c12",   # Orange
    "Rare": "#e74c3c",   # Red
}


class KdfVisualizer:
    """
    Interactive visualization for KDF results.

    Provides various visualization methods for exploring KDF outputs
    in Jupyter notebooks.
    """

    def __init__(self, use_plotly: bool = True):
        """
        Initialize visualizer.

        Args:
            use_plotly: Use Plotly for interactive plots (default True)
        """
        self.use_plotly = use_plotly and HAS_PLOTLY

        if not HAS_NUMPY:
            raise ImportError("numpy is required: pip install numpy")

    def plot_layers(
        self,
        result: "KdfResult",
        items: Optional[List[Any]] = None,
        title: str = "KDF Layer Distribution",
        figsize: Tuple[int, int] = (10, 6),
    ):
        """
        Plot layer distribution as a bar chart.

        Args:
            result: KDF result
            items: Original items (optional, for hover text)
            title: Plot title
            figsize: Figure size (matplotlib only)
        """
        # Count layers
        layer_counts = {"Core": 0, "Edge": 0, "Rare": 0}
        for layer in result.layers:
            layer_name = str(layer)
            if layer_name in layer_counts:
                layer_counts[layer_name] += 1

        if self.use_plotly:
            fig = go.Figure(data=[
                go.Bar(
                    x=list(layer_counts.keys()),
                    y=list(layer_counts.values()),
                    marker_color=[LAYER_COLORS[k] for k in layer_counts.keys()],
                    text=list(layer_counts.values()),
                    textposition='auto',
                )
            ])
            fig.update_layout(
                title=title,
                xaxis_title="Layer",
                yaxis_title="Count",
                showlegend=False,
            )
            fig.show()
        else:
            if not HAS_MATPLOTLIB:
                raise ImportError("matplotlib is required: pip install matplotlib")

            fig, ax = plt.subplots(figsize=figsize)
            bars = ax.bar(
                layer_counts.keys(),
                layer_counts.values(),
                color=[LAYER_COLORS[k] for k in layer_counts.keys()],
            )
            ax.set_xlabel("Layer")
            ax.set_ylabel("Count")
            ax.set_title(title)

            # Add value labels
            for bar in bars:
                height = bar.get_height()
                ax.annotate(
                    f'{height}',
                    xy=(bar.get_x() + bar.get_width() / 2, height),
                    ha='center', va='bottom'
                )
            plt.tight_layout()
            plt.show()

    def plot_selection_pie(
        self,
        result: "KdfResult",
        title: str = "Selection Summary",
    ):
        """
        Plot selection summary as a pie chart.

        Args:
            result: KDF result
            title: Plot title
        """
        total = len(result.layers)
        selected = len(result.selected)
        filtered = total - selected

        if self.use_plotly:
            fig = go.Figure(data=[
                go.Pie(
                    labels=["Selected", "Filtered"],
                    values=[selected, filtered],
                    marker_colors=["#2ecc71", "#95a5a6"],
                    hole=0.3,
                    textinfo='label+percent+value',
                )
            ])
            fig.update_layout(title=title)
            fig.show()
        else:
            if not HAS_MATPLOTLIB:
                raise ImportError("matplotlib is required")

            fig, ax = plt.subplots(figsize=(8, 6))
            ax.pie(
                [selected, filtered],
                labels=["Selected", "Filtered"],
                colors=["#2ecc71", "#95a5a6"],
                autopct='%1.1f%%',
                startangle=90,
            )
            ax.set_title(title)
            plt.tight_layout()
            plt.show()

    def plot_2d_scatter(
        self,
        result: "KdfResult",
        vectors: List[List[float]],
        title: str = "KDF 2D Visualization",
        reduce_dims: bool = True,
    ):
        """
        Plot items in 2D space colored by layer.

        Args:
            result: KDF result
            vectors: Item vectors (will be reduced if > 2D)
            title: Plot title
            reduce_dims: Use PCA to reduce dimensions if needed
        """
        if not HAS_NUMPY:
            raise ImportError("numpy is required")

        vectors = np.array(vectors)
        n_dims = vectors.shape[1] if len(vectors.shape) > 1 else 1

        # Reduce dimensions if needed
        if n_dims > 2 and reduce_dims:
            try:
                from sklearn.decomposition import PCA
                pca = PCA(n_components=2)
                vectors_2d = pca.fit_transform(vectors)
            except ImportError:
                # Simple fallback: use first two dimensions
                vectors_2d = vectors[:, :2]
        elif n_dims == 1:
            vectors_2d = np.column_stack([vectors, np.zeros(len(vectors))])
        else:
            vectors_2d = vectors[:, :2]

        # Get layer colors
        colors = []
        for layer in result.layers:
            layer_name = str(layer)
            colors.append(LAYER_COLORS.get(layer_name, "#7f8c8d"))

        # Determine selected vs not selected
        selected_set = set(result.selected)
        markers = ["x" if i not in selected_set else "o" for i in range(len(vectors))]

        if self.use_plotly:
            # Create traces for each layer
            traces = []
            for layer_name, color in LAYER_COLORS.items():
                mask = [str(result.layers[i]) == layer_name for i in range(len(result.layers))]
                if not any(mask):
                    continue

                x_vals = [vectors_2d[i, 0] for i in range(len(mask)) if mask[i]]
                y_vals = [vectors_2d[i, 1] for i in range(len(mask)) if mask[i]]
                indices = [i for i in range(len(mask)) if mask[i]]
                selected_status = ["Selected" if i in selected_set else "Filtered" for i in indices]

                traces.append(go.Scatter(
                    x=x_vals,
                    y=y_vals,
                    mode='markers',
                    name=layer_name,
                    marker=dict(
                        color=color,
                        size=[12 if i in selected_set else 8 for i in indices],
                        symbol=['circle' if i in selected_set else 'x' for i in indices],
                    ),
                    text=[f"Index: {i}<br>Status: {s}" for i, s in zip(indices, selected_status)],
                    hoverinfo='text+name',
                ))

            fig = go.Figure(data=traces)
            fig.update_layout(
                title=title,
                xaxis_title="Dimension 1",
                yaxis_title="Dimension 2",
            )
            fig.show()
        else:
            if not HAS_MATPLOTLIB:
                raise ImportError("matplotlib is required")

            fig, ax = plt.subplots(figsize=(10, 8))

            for layer_name, color in LAYER_COLORS.items():
                mask = [str(result.layers[i]) == layer_name for i in range(len(result.layers))]
                if not any(mask):
                    continue

                x_vals = [vectors_2d[i, 0] for i in range(len(mask)) if mask[i]]
                y_vals = [vectors_2d[i, 1] for i in range(len(mask)) if mask[i]]
                indices = [i for i in range(len(mask)) if mask[i]]

                ax.scatter(
                    x_vals, y_vals,
                    c=color,
                    label=layer_name,
                    s=[100 if i in selected_set else 30 for i in indices],
                    marker='o',
                    alpha=0.7,
                )

            ax.set_xlabel("Dimension 1")
            ax.set_ylabel("Dimension 2")
            ax.set_title(title)
            ax.legend()
            plt.tight_layout()
            plt.show()

    def plot_threshold_analysis(
        self,
        kdf: "Kdf",
        items: List[Any],
        similarity_fn: str = "cosine",
        thresholds: Optional[List[float]] = None,
        title: str = "Threshold Analysis",
    ):
        """
        Plot how results change with different thresholds.

        Args:
            kdf: KDF instance
            items: Items to process
            similarity_fn: "cosine", "euclidean", or "levenshtein"
            thresholds: Thresholds to evaluate (default: 0.5 to 0.95)
            title: Plot title
        """
        if thresholds is None:
            thresholds = [0.50, 0.55, 0.60, 0.65, 0.70, 0.75, 0.80, 0.85, 0.90, 0.95]

        results_data = {
            "threshold": [],
            "selected": [],
            "compression": [],
            "core": [],
            "edge": [],
            "rare": [],
        }

        for threshold in thresholds:
            if similarity_fn == "cosine":
                result = kdf.process_vectors(items, threshold=threshold)
            elif similarity_fn == "euclidean":
                result = kdf.process_euclidean(items, threshold=threshold)
            else:
                result = kdf.process_text(items, threshold=threshold)

            results_data["threshold"].append(threshold)
            results_data["selected"].append(len(result.selected))
            results_data["compression"].append(result.compression_rate() * 100)
            results_data["core"].append(len(result.core_items()))
            results_data["edge"].append(len(result.edge_items()))
            results_data["rare"].append(len(result.rare_items()))

        if self.use_plotly:
            fig = make_subplots(
                rows=2, cols=2,
                subplot_titles=["Selected Count", "Compression Rate (%)", "Layer Distribution", "Rare Items"],
            )

            # Selected count
            fig.add_trace(
                go.Scatter(x=results_data["threshold"], y=results_data["selected"],
                          mode='lines+markers', name="Selected"),
                row=1, col=1
            )

            # Compression rate
            fig.add_trace(
                go.Scatter(x=results_data["threshold"], y=results_data["compression"],
                          mode='lines+markers', name="Compression %"),
                row=1, col=2
            )

            # Layer distribution
            fig.add_trace(
                go.Scatter(x=results_data["threshold"], y=results_data["core"],
                          mode='lines+markers', name="Core", line=dict(color=LAYER_COLORS["Core"])),
                row=2, col=1
            )
            fig.add_trace(
                go.Scatter(x=results_data["threshold"], y=results_data["edge"],
                          mode='lines+markers', name="Edge", line=dict(color=LAYER_COLORS["Edge"])),
                row=2, col=1
            )
            fig.add_trace(
                go.Scatter(x=results_data["threshold"], y=results_data["rare"],
                          mode='lines+markers', name="Rare", line=dict(color=LAYER_COLORS["Rare"])),
                row=2, col=1
            )

            # Rare items (important!)
            fig.add_trace(
                go.Scatter(x=results_data["threshold"], y=results_data["rare"],
                          mode='lines+markers', name="Rare", fill='tozeroy',
                          line=dict(color=LAYER_COLORS["Rare"])),
                row=2, col=2
            )

            fig.update_layout(height=600, title_text=title, showlegend=True)
            fig.show()
        else:
            if not HAS_MATPLOTLIB:
                raise ImportError("matplotlib is required")

            fig, axes = plt.subplots(2, 2, figsize=(12, 10))

            # Selected count
            axes[0, 0].plot(results_data["threshold"], results_data["selected"], 'b-o')
            axes[0, 0].set_xlabel("Threshold")
            axes[0, 0].set_ylabel("Selected Count")
            axes[0, 0].set_title("Selected Count")

            # Compression rate
            axes[0, 1].plot(results_data["threshold"], results_data["compression"], 'g-o')
            axes[0, 1].set_xlabel("Threshold")
            axes[0, 1].set_ylabel("Compression %")
            axes[0, 1].set_title("Compression Rate (%)")

            # Layer distribution
            axes[1, 0].plot(results_data["threshold"], results_data["core"], '-o',
                          color=LAYER_COLORS["Core"], label="Core")
            axes[1, 0].plot(results_data["threshold"], results_data["edge"], '-o',
                          color=LAYER_COLORS["Edge"], label="Edge")
            axes[1, 0].plot(results_data["threshold"], results_data["rare"], '-o',
                          color=LAYER_COLORS["Rare"], label="Rare")
            axes[1, 0].set_xlabel("Threshold")
            axes[1, 0].set_ylabel("Count")
            axes[1, 0].set_title("Layer Distribution")
            axes[1, 0].legend()

            # Rare items
            axes[1, 1].fill_between(results_data["threshold"], results_data["rare"],
                                   color=LAYER_COLORS["Rare"], alpha=0.3)
            axes[1, 1].plot(results_data["threshold"], results_data["rare"], '-o',
                          color=LAYER_COLORS["Rare"])
            axes[1, 1].set_xlabel("Threshold")
            axes[1, 1].set_ylabel("Rare Count")
            axes[1, 1].set_title("Rare Items (Important!)")

            fig.suptitle(title)
            plt.tight_layout()
            plt.show()

    def interactive_threshold(
        self,
        kdf: "Kdf",
        items: List[Any],
        similarity_fn: str = "cosine",
        initial_threshold: float = 0.8,
    ):
        """
        Create an interactive threshold slider widget.

        Args:
            kdf: KDF instance
            items: Items to process
            similarity_fn: "cosine", "euclidean", or "levenshtein"
            initial_threshold: Initial threshold value
        """
        if not HAS_WIDGETS:
            raise ImportError("ipywidgets is required: pip install ipywidgets")

        output = widgets.Output()

        def update(threshold):
            with output:
                output.clear_output(wait=True)

                if similarity_fn == "cosine":
                    result = kdf.process_vectors(items, threshold=threshold)
                elif similarity_fn == "euclidean":
                    result = kdf.process_euclidean(items, threshold=threshold)
                else:
                    result = kdf.process_text(items, threshold=threshold)

                # Display summary
                summary = f"""
                <div style="padding: 10px; background: #f8f9fa; border-radius: 5px; margin-bottom: 10px;">
                    <h3>Threshold: {threshold:.2f}</h3>
                    <p><strong>Total items:</strong> {len(items)}</p>
                    <p><strong>Selected:</strong> {len(result.selected)} ({100-result.compression_rate()*100:.1f}%)</p>
                    <p><strong>Compression:</strong> {result.compression_rate()*100:.1f}%</p>
                    <hr>
                    <p style="color: {LAYER_COLORS['Core']}"><strong>Core:</strong> {len(result.core_items())}</p>
                    <p style="color: {LAYER_COLORS['Edge']}"><strong>Edge:</strong> {len(result.edge_items())}</p>
                    <p style="color: {LAYER_COLORS['Rare']}"><strong>Rare:</strong> {len(result.rare_items())}</p>
                </div>
                """
                display(HTML(summary))

                # Plot layers
                self.plot_layers(result, title=f"Layer Distribution (threshold={threshold:.2f})")

        # Create slider
        slider = widgets.FloatSlider(
            value=initial_threshold,
            min=0.3,
            max=0.99,
            step=0.01,
            description='Threshold:',
            continuous_update=False,
            style={'description_width': 'initial'},
            layout=widgets.Layout(width='80%'),
        )

        # Create interactive widget
        interactive = widgets.interactive(update, threshold=slider)

        # Display
        display(widgets.VBox([
            widgets.HTML("<h2>KDF Interactive Threshold Explorer</h2>"),
            slider,
            output,
        ]))

    def summary_table(self, result: "KdfResult") -> str:
        """
        Generate a text summary table.

        Args:
            result: KDF result

        Returns:
            Formatted summary string
        """
        total = len(result.layers)
        selected = len(result.selected)
        core = len(result.core_items())
        edge = len(result.edge_items())
        rare = len(result.rare_items())

        table = f"""
+-------------------+--------+
| Metric            | Value  |
+-------------------+--------+
| Total Items       | {total:>6} |
| Selected          | {selected:>6} |
| Compression       | {result.compression_rate()*100:>5.1f}% |
+-------------------+--------+
| Core Items        | {core:>6} |
| Edge Items        | {edge:>6} |
| Rare Items        | {rare:>6} |
+-------------------+--------+
"""
        return table


def demo():
    """Demo function for visualization."""
    if not HAS_KDF_RS:
        print("kdf_rs not available. Build with: cd kdf-python && maturin develop --release")
        return

    if not HAS_NUMPY:
        print("numpy is required: pip install numpy")
        return

    print("=== KDF Visualization Demo ===\n")

    # Create sample data
    np.random.seed(42)

    # Cluster 1
    cluster1 = np.random.randn(20, 2) * 0.3 + np.array([0, 0])
    # Cluster 2
    cluster2 = np.random.randn(20, 2) * 0.3 + np.array([3, 0])
    # Cluster 3
    cluster3 = np.random.randn(10, 2) * 0.3 + np.array([1.5, 3])
    # Outliers
    outliers = np.array([
        [5, 5],
        [-3, 4],
        [4, -3],
    ])

    vectors = np.vstack([cluster1, cluster2, cluster3, outliers]).tolist()

    # Process with KDF
    kdf = Kdf()
    result = kdf.process_vectors(vectors, threshold=0.8)

    # Create visualizer
    viz = KdfVisualizer(use_plotly=HAS_PLOTLY)

    # Print summary
    print(viz.summary_table(result))

    # Plot layers
    print("Plotting layer distribution...")
    viz.plot_layers(result, title="Sample Data - Layer Distribution")

    # Plot 2D scatter
    print("\nPlotting 2D scatter...")
    viz.plot_2d_scatter(result, vectors, title="Sample Data - 2D View")

    print("\nDone! In Jupyter, you can use viz.interactive_threshold() for interactive exploration.")


if __name__ == "__main__":
    demo()
