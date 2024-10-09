use crate::assembler::assemble;

mod assembler;

fn main() {
    let prog = "j 899";
    let bin = assemble(prog);
    println!("{:?}", bin);
    //println!("compiled! :D");
}
