pub mod colorscheme;
pub mod dissolve;
pub mod labeler;
pub mod map_type;
pub mod projection;
pub mod renderer;
pub mod rounds;

pub use colorscheme::{
    graph_color, CategoricalScheme, CompactnessScheme, DemographicScheme, PoliticalScheme,
};
pub use dissolve::{dissolve_geometries, group_dissolve, wkb_to_geometry};
pub use labeler::{
    adaptive_font_size, compactness_label, demographic_label, halo_text_svg, label_fits,
    largest_component, political_label, round_label, round_label_with_lineage, LabelSpec,
};
pub use projection::{InsetProjection, Projection};
pub use renderer::{build_svg, canvas_size_from_dpi, default_font_db, svg_to_png};
