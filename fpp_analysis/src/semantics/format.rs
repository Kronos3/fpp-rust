#[derive(Debug, Clone)]
pub struct FormatField {}

#[derive(Debug, Clone)]
pub struct Format {
    /** The first part of the format, before any fields */
    pub prefix: String,
    /** The list of pairs of fields followed by suffix strings */
    pub fields: Vec<(FormatField, String)>,
}
