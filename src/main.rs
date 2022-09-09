use diazo::{lexer, filehandling, parser};

fn main() {
    let test = lexer::lexer(filehandling::read_file(&"prokaryotes.dz").expect("oops")).expect("you fucked up");
    dbg!(&test);
    let test2 = parser::parser(test).unwrap();
    for i in test2 {
        println!("{}", i.print());
    }
}
