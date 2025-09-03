//! Emulator module for RV64

#[derive(Debug)]
pub struct Emu {
    cpu: (),
    mem: (),
    bus: ()
}


impl Emu {
    pub fn new() -> Self {
        Emu {
            cpu: (),
            mem: (),
            bus: ()
        }
    }
    
    pub fn restart(&mut self) {
        self.cpu = ();
        self.mem = ();
        self.bus = ();
    }
    
    pub fn run(&mut self) {
        
    }
}
