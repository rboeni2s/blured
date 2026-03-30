use serde::de::Visitor;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum Color
{
    Rgb(RgbColor),
    Hex(HexColor),
}


impl Default for Color
{
    fn default() -> Self
    {
        Color::Hex(HexColor(0x071019))
    }
}


impl From<Color> for [f32; 3]
{
    fn from(val: Color) -> Self
    {
        match val
        {
            Color::Rgb(rgb_color) => rgb_color.into(),
            Color::Hex(hex_color) => hex_color.into(),
        }
    }
}


#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RgbColor(u8, u8, u8);


impl From<RgbColor> for [f32; 3]
{
    fn from(val: RgbColor) -> Self
    {
        [
            val.0 as f32 / 255.0,
            val.1 as f32 / 255.0,
            val.2 as f32 / 255.0,
        ]
    }
}


#[derive(Debug, Copy, Clone)]
pub struct HexColor(u32);


impl From<HexColor> for [f32; 3]
{
    fn from(val: HexColor) -> Self
    {
        let rgb = &val.0.to_be_bytes()[1..];
        [
            rgb[0] as f32 / 255.0,
            rgb[1] as f32 / 255.0,
            rgb[2] as f32 / 255.0,
        ]
    }
}


struct HexStringVisitor;
impl<'de> Visitor<'de> for HexStringVisitor
{
    type Value = u32;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        formatter.write_str("an rgb hex color representation optionally prefixed by a '#' or '0x'")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let lower = v.to_lowercase();
        let trimmed = lower.trim_start_matches("0x").trim_start_matches("#");
        u32::from_str_radix(trimmed, 16).map_err(|e| E::custom(e.to_string()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(&v)
    }
}


impl<'de> serde::Deserialize<'de> for HexColor
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(HexStringVisitor).map(HexColor)
    }
}


impl serde::Serialize for HexColor
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("#{:X}", self.0))
    }
}
