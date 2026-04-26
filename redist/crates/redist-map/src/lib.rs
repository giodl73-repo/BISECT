pub mod colorscheme;
pub mod dissolve;
pub mod labeler;
pub mod map_type;
pub mod projection;
pub mod renderer;
pub mod rounds;

pub use projection::Projection;
pub use colorscheme::{CategoricalScheme, PoliticalScheme, DemographicScheme, CompactnessScheme, graph_color};
pub use labeler::{LabelSpec, adaptive_font_size, label_fits, round_label, round_label_with_lineage,
                  political_label, demographic_label, compactness_label, halo_text_svg, largest_component};
pub use dissolve::{wkb_to_geometry, dissolve_geometries, group_dissolve};
pub use renderer::{build_svg, svg_to_png, default_font_db, canvas_size_from_dpi};
