use crate::assembler::assemble;

mod assembler;

fn main() {
    let prog = "sys 899\njr 8\n\n\n\n\n\n\n";
    let bin = assemble(prog);
    println!("{:?}", bin);
    //println!("compiled! :D");
}
