use necsim_core::{
    event::{DispersalEvent, SpeciationEvent},
    impl_report,
    reporter::Reporter,
};

#[allow(clippy::module_name_repetitions)]
pub struct EventReporter {
    last_speciation_event: Option<SpeciationEvent>,
    last_dispersal_event: Option<DispersalEvent>,

    speciation: usize,
    out_dispersal: usize,
    self_dispersal: usize,
    out_coalescence: usize,
    self_coalescence: usize,
}

impl Reporter for EventReporter {
    impl_report!(speciation(&mut self, event: Unused) -> Used {
        event.use_in(|event| {
            if Some(event) == self.last_speciation_event.as_ref() {
                return;
            }
            self.last_speciation_event = Some(event.clone());

            self.speciation += 1;
        })
    });

    impl_report!(dispersal(&mut self, event: Unused) -> Used {
        event.use_in(|event| {
            if Some(event) == self.last_dispersal_event.as_ref() {
                return;
            }
            self.last_dispersal_event = Some(event.clone());

            let self_dispersal = event.origin == event.target;
            let coalescence = event.coalescence.is_some();

            match (self_dispersal, coalescence) {
                (true, true) => {
                    self.self_coalescence += 1;
                },
                (true, false) => {
                    self.self_dispersal += 1;
                },
                (false, true) => {
                    self.out_coalescence += 1;
                },
                (false, false) => {
                    self.out_dispersal += 1;
                },
            }
        })
    });

    impl_report!(progress(&mut self, remaining: Unused) -> Unused {
        remaining.ignore()
    });
}

impl Default for EventReporter {
    #[debug_ensures(
        ret.speciation == 0 &&
        ret.out_dispersal == 0 &&
        ret.self_dispersal == 0 &&
        ret.out_coalescence == 0 &&
        ret.self_coalescence == 0,
        "initialises all events to 0"
    )]
    fn default() -> Self {
        Self {
            last_speciation_event: None,
            last_dispersal_event: None,

            speciation: 0,
            out_dispersal: 0,
            self_dispersal: 0,
            out_coalescence: 0,
            self_coalescence: 0,
        }
    }
}

impl EventReporter {
    pub fn report(self) {
        println!("{:=^80}", " Event Summary ");

        println!(
            "Total #individuals:\n\t{}",
            self.speciation + self.self_coalescence + self.out_coalescence
        );
        println!(
            "Total #events:\n\t{}",
            self.speciation
                + self.self_coalescence
                + self.out_coalescence
                + self.self_dispersal
                + self.out_dispersal
        );

        println!("Speciation:\n\t{}", self.speciation);
        println!("Dispersal outside cell:\n\t{}", self.out_dispersal);
        println!("Dispersal inside cell:\n\t{}", self.self_dispersal);
        println!("Coalescence outside cell:\n\t{}", self.out_coalescence);
        println!("Coalescence inside cell:\n\t{}", self.self_coalescence);

        println!("{:=^80}", " Event Summary ");
    }
}