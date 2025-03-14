#[derive(Debug)]
pub struct candle_stick {
    pub source: String,
    pub ts: u64,
    pub o: f64,
    pub h: f64,
    pub l: f64,
    pub c: f64,
    pub v: f64,
}

pub struct CandleStickBuilder {
    source: String,
    ts: u64,
    o: f64,
    h: f64,
    l: f64,
    c: f64,
    v: f64,
}

impl CandleStickBuilder {
    // Constructor for the builder with default values
    pub fn new() -> Self {
        CandleStickBuilder {
            source: String::from(""),
            ts: 0,  // Default to 0
            o: 0.0, // Default to 0.0
            h: 0.0, // Default to 0.0
            l: 0.0, // Default to 0.0
            c: 0.0, // Default to 0.0
            v: 0.0, // Default to 0.0
        }
    }

    // Methods to set each field (can be used to override default values)
    pub fn source(mut self, source: String) -> Self {
        self.source = source;
        self
    }

    pub fn ts(mut self, ts: u64) -> Self {
        self.ts = ts;
        self
    }

    pub fn o(mut self, o: f64) -> Self {
        self.o = o;
        self
    }

    pub fn h(mut self, h: f64) -> Self {
        self.h = h;
        self
    }

    pub fn l(mut self, l: f64) -> Self {
        self.l = l;
        self
    }

    pub fn c(mut self, c: f64) -> Self {
        self.c = c;
        self
    }

    pub fn v(mut self, v: f64) -> Self {
        self.v = v;
        self
    }

    // Build the final candle_stick struct
    pub fn build(self) -> candle_stick {
        candle_stick {
            source: self.source,
            ts: self.ts,
            o: self.o,
            h: self.h,
            l: self.l,
            c: self.c,
            v: self.v,
        }
    }
}

