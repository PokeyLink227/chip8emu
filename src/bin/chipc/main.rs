use crate::assembler::assemble;

mod assembler;

fn main() {
    let prog = "
    mov v1, 5
    ";
    let bin = assemble(prog);
    println!("{:?}", bin);
    //println!("compiled! :D");
}
