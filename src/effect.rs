use rand::rngs::StdRng;

pub struct ParamDesc {
    pub name: String,
    pub min: f64,
    pub max: f64,
    pub value: f64,
}

pub trait Effect {
    fn name(&self) -> &str;
    fn init(&mut self, width: u32, height: u32);
    fn randomize_init(&mut self, _rng: &mut StdRng) {}
    fn update(&mut self, t: f64, dt: f64, pixels: &mut [(u8, u8, u8)]);
    fn cleanup(&mut self) {}
    fn params(&self) -> Vec<ParamDesc> {
        vec![]
    }
    fn set_param(&mut self, _name: &str, _value: f64) {}
}
