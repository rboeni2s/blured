use serde::de::Visitor;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum Color
{
    // allow non_camel_case because the variant names are used by the toml config
    #[allow(non_camel_case_types)]
    rgb(RgbColor),
    #[allow(non_camel_case_types)]
    hex(HexColor),
}


impl Default for Color
{
    fn default() -> Self
    {
        Color::hex(HexColor(0x0E1F33))
    }
}


impl Into<[f32; 3]> for Color
{
    fn into(self) -> [f32; 3]
    {
        match self
        {
            Color::rgb(rgb_color) => rgb_color.into(),
            Color::hex(hex_color) => hex_color.into(),
        }
    }
}


#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct RgbColor(u8, u8, u8);


impl Into<[f32; 3]> for RgbColor
{
    fn into(self) -> [f32; 3]
    {
        [
            self.0 as f32 / 255.0,
            self.1 as f32 / 255.0,
            self.2 as f32 / 255.0,
        ]
    }
}


#[derive(Debug, Copy, Clone)]
pub struct HexColor(u32);


impl Into<[f32; 3]> for HexColor
{
    fn into(self) -> [f32; 3]
    {
        let rgb = &self.0.to_le_bytes()[1..];
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
        deserializer
            .deserialize_str(HexStringVisitor)
            .map(|c| HexColor(c))
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
