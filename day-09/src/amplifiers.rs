use std::thread;
use std::sync::mpsc::{ self, Sender, Receiver };

#[path = "computer/mod.rs"]
mod computer;
use computer::Computer as Computer;

struct Amplifier {
    phase: i64,
    tx: Option::<Sender::<i64>>,
    rx: Option::<Receiver::<i64>>,
}

impl Amplifier {
    fn new(phase: i64) -> Amplifier {
        Amplifier { phase, tx: None, rx: None }
    }

    fn set_tx(&mut self, tx: Sender::<i64>) {
        self.tx = Some(tx);
    }

    fn set_rx(&mut self, rx: Receiver::<i64>) {
        self.rx = Some(rx);
    }

    fn run(&mut self, program: Vec<i64>, out_tx: Option<Sender<i64>>) -> thread::JoinHandle<()> {
        let phase = self.phase;
        let tx = self.tx.take().unwrap();
        let rx = self.rx.take().unwrap();

        thread::spawn(move || {
            let mut phase_queried = false;

            let input = || {
                if !phase_queried {
                    phase_queried = true;
                    phase
                } else {
                    rx.recv().unwrap()
                }
            };

            let output = |out_signal: i64| {
                // This will fail for the last output of the last amplifier, since the first one
                // will have closed the channel
                let _ = tx.send(out_signal);

                if let Some(out) = out_tx.as_ref() {
                    out.send(out_signal).unwrap();
                }
            };

            let mut computer = Computer::new(program, input, output);
            computer.run();
        })
    }
}

pub fn run_phase_sequence(program: Vec<i64>, phase_sequence: Vec<i64>) -> i64 {
    let mut amplifiers = setup_amplifiers(phase_sequence);

    // Clone the tx for the channel the first amplifier reads from
    let tx_start = amplifiers[amplifiers.len() - 1].tx.clone().unwrap();

    let mut threads = Vec::<thread::JoinHandle<()>>::new();

    let total_amplifiers = amplifiers.len();

    // The first n-1 amplifiers are started regularly
    for mut amplifier in amplifiers.drain(0..total_amplifiers-1) {
        threads.push(amplifier.run(program.clone(), None));
    }

    // Create a new channel from the last amplifier to the outside world
    let (tx, rx) = mpsc::channel();

    // The last amplifier is fed the tx to the channel we receive from
    assert_eq!(amplifiers.len(), 1);
    threads.push(amplifiers[0].run(program.clone(), Some(tx)));

    // Kickstart process by sending initial signal to the first amplifier
    tx_start.send(0).unwrap();

    // Store received values
    let mut values = Vec::<i64>::new();
    for received in rx {
        values.push(received);
    }

    // All threads should've finished by now (since the last one closed the channel), so this
    // shouldn't be necessary
    for thread in threads {
        thread.join().unwrap();
    }

    // Return the last received value
    *values.iter().last().unwrap()
}

fn setup_amplifiers(phase_sequence: Vec<i64>) -> Vec<Amplifier> {
    let mut amplifiers = phase_sequence
        .iter()
        .map(|phase| Amplifier::new(*phase))
        .collect::<Vec<Amplifier>>();

    let total_amplifiers = amplifiers.len();

    for i in 0..total_amplifiers {
        let (tx, rx) = mpsc::channel();

        // Connect each amplifier's tx to the next one's rx
        amplifiers[i].set_tx(tx);
        amplifiers[(i + 1) % total_amplifiers].set_rx(rx);
    }

    amplifiers
}

#[cfg(test)]
mod sequencer {
    use super::*;

    #[test]
    fn test_basic_sequences() {
        assert_eq!(
            run_phase_sequence(
                vec![3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0],
                vec![4,3,2,1,0]
            ), 43210
        );

        assert_eq!(
            run_phase_sequence(
                vec![
                    3,23,3,24,1002,24,10,24,1002,23,-1,23,
                    101,5,23,23,1,24,23,23,4,23,99,0,0
                ],
                vec![0,1,2,3,4]
            ), 54321
        );

        assert_eq!(
            run_phase_sequence(
                vec![
                    3,31,3,32,1002,32,10,32,1001,31,-2,31,1007,31,0,33,
                    1002,33,7,33,1,33,31,31,1,32,31,31,4,31,99,0,0,0
                ],
                vec![1,0,4,3,2]
            ), 65210
        );
    }

    #[test]
    fn test_looped_sequences() {
        assert_eq!(
            run_phase_sequence(
                vec![
                    3,26,1001,26,-4,26,3,27,1002,27,2,27,1,27,26,
                    27,4,27,1001,28,-1,28,1005,28,6,99,0,0,5
                ],
                vec![9,8,7,6,5]
            ), 139629729
        );

        assert_eq!(
            run_phase_sequence(
                vec![
                    3,52,1001,52,-5,52,3,53,1,52,56,54,1007,54,5,55,1005,55,26,1001,54,
                    -5,54,1105,1,12,1,53,54,53,1008,54,0,55,1001,55,1,55,2,53,55,53,4,
                    53,1001,56,-1,56,1005,56,6,99,0,0,0,0,10
                ],
                vec![9,7,8,5,6]
            ), 18216
        );
    }
}
