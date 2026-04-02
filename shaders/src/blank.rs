use crate::*;


// Blank effect, render the background image as is
effect!(|Effect { tex, .. }, texture, sampler| { texture.sample(*sampler, tex).xyz() });
