use std::process::{Command, Output};
use std::io;

use std::thread;

#[derive(Debug)]
enum Wait {
    ForAll,
    ForAny,
}

impl std::str::FromStr for Wait {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
        match s {
            "for-all" | "all" => Ok(Wait::ForAll),
            "for-any" | "any" => Ok(Wait::ForAny),
            _ => { Err("cannot convert from value".into()) }
        }
    }
}

#[derive(Debug)]
enum OnError {
    Interrupt,
    Ignore,
}

impl std::str::FromStr for OnError {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
        match s {
            "int" | "interrupt" | "break" | "stop" => Ok(OnError::Interrupt),
            "ignore" => Ok(OnError::Ignore),
            _ => { Err("cannot convert from value".into()) }
        }
    }
}


#[derive(Debug)]
struct ProcFinished {
    res: Result<Output, io::Error>,
}

impl ProcFinished {
    fn must_success(&self) {
        match &self.res {
            Ok(o) => {
                if !o.status.success() {
                    panic!("Error");
                }
            },
            Err(e) => {
                todo!();
            }
        }
    }
}

struct Proc {
    command: String,
    do_wait: bool,
    shell: String,
}

impl Proc {
    fn new(command: &str, do_wait: bool, shell: &str) -> Self {
        Self {
            command: command.into(),
            do_wait,
            shell: shell.into(),
        }
    }

    // fn eval(&self) -> Result<Output, io::Error> {
    //     let output = Command::new(&self.shell)
    //         .arg("-c")
    //         .arg(&self.command)
    //         .output();

    //     output
    // }

    fn eval(self) -> ProcFinished {
        let output = Command::new(&self.shell)
            .arg("-c")
            .arg(&self.command)
            .output();

        ProcFinished { res: output }
    }
}

#[derive(Default)]
struct ProcBuilder {
    command: Option<String>,
    do_wait: Option<bool>,
    shell: Option<String>,
}

impl ProcBuilder {
    fn new() -> Self {
        Self { .. Default::default() }
    }
    fn command(mut self, command: &str) -> Self {
        self.command = Some(command.into());
        self
    }

    fn wait(mut self) -> Self {
        self.do_wait = Some(true);
        self
    }

    fn shell(mut self, shell: &str) -> Self {
        self.shell = Some(shell.into());
        self
    }

    fn build(self) -> Proc {
        match self.command {
            Some(c) => {
                Proc {
                    command: c,
                    do_wait: self.do_wait.unwrap_or(false),
                    shell: self.shell.unwrap_or("bash".into()),
                }
            },
            None => {
                todo!();
            }
        }
    }
}

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "parallel", about = "Simple parallel")]
struct Opt {
    #[structopt(short, long)]
    debug: bool,

    #[structopt(long)]
    wait: Wait,

    #[structopt(long = "on-error")]
    on_error: OnError,

    #[structopt(parse(try_from_str))]
    args: Vec<String>,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt.args);

    let mut handles = vec![];

    for c in opt.args {
        println!("> {}", c);
        let h = thread::spawn(move || {
            let pb = ProcBuilder::new()
                .command(&c)
                .wait();
            let proc = pb.build();
            let r = proc.eval();
            println!("r: {:#?}", r);
        });

        handles.push(h);
    }

    for h in handles {
        h.join().unwrap();
    }

    println!("Joined.");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn smoke_bash() {
        let output = Command::new("bash")
            .arg("-c")
            .arg("echo Hello world")
            .output()
            .expect("Failed to execute command");

        assert_eq!(b"Hello world\n", output.stdout.as_slice());
    }

    #[test]
    fn smoke_eval() {
        let proc = Proc::new("echo hello world", true, "bash");
        // assert!(proc.eval().is_ok());
        proc.eval();
    }

    #[test]
    fn smoke_builder() {
        let pb = ProcBuilder::new()
            .command("echo hello world")
            .wait();
        let proc = pb.build();
        assert_eq!(proc.shell, String::from("bash"));
        proc.eval();
    }

    #[test]
    fn smoke_builder_shell() {
        let pb = ProcBuilder::new()
            .command("echo hello world")
            .wait()
            .shell("zsh");
        let proc = pb.build();
        assert_eq!(proc.shell, String::from("zsh"));
    }
    
    #[test]
    fn smoke_finished_success() {
        let pb = ProcBuilder::new()
            .command("echo hello world")
            .wait()
            .shell("bash");
        let proc = pb.build().eval();
        proc.must_success();
    }

    #[test]
    #[should_panic]
    fn smoke_finished_failure() {
        let pb = ProcBuilder::new()
            .command("false")
            .wait()
            .shell("bash");
        let proc = pb.build().eval();
        proc.must_success();
    }
}
