pub struct City {
    pub name: &'static str,
    pub lat: f64,
    pub lon: f64,
}

pub const CITIES: &[City] = &[
    City {
        name: "Phoenix, AZ",
        lat: 33.4484,
        lon: -112.0740,
    },
    City {
        name: "Lahore, Pakistan",
        lat: 31.5204,
        lon: 74.3587,
    },
    City {
        name: "Baghdad, Iraq",
        lat: 33.3152,
        lon: 44.3661,
    },
    // add more cities as needed...
];
