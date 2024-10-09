use crate::assembler::Instr::*;
use regex::Regex;

#[derive(Debug)]
enum Instr {
    CallMachineCode(u16),
    ClearScreen,
    Return,
    Jump(u16),
    Call(u16),
    IfEqualImm(u8, u8),
    IfNotEqualImm(u8, u8),
    IfEqualReg(u8, u8),
    SetImm(u8, u8),
    AddImm(u8, u8),
    SetReg(u8, u8),
    OrReg(u8, u8),
    AndReg(u8, u8),
    XorReg(u8, u8),
    AddReg(u8, u8),
    SubReg(u8, u8),
    ShiftRight(u8, u8),
    SetSubReg(u8, u8),
    ShiftLeft(u8, u8),
    IfNotEqualReg(u8, u8),
    SetI(u16),
    JumpReg(u16),
    Rand(u8, u8),
    Draw(u8, u8, u8),
    IfKey(u8),
    IfNotKey(u8),
    GetTimer(u8),
    GetKey(u8),
    SetTimer(u8),
    SetSound(u8),
    AddIReg(u8),
    SetICharAddr(u8),
    StoreDecimal(u8),
    StoreReg(u8),
    LoadReg(u8),
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Token {
    Clear,
    Return,
    Jump,
    Call,
    BranchEqual,
    BranchNotEqual,
    Move,
    Add,
    Or,
    And,
    Xor,
    Sub,
    ShiftRight,
    ShiftLeft,
    JumpReg,
    Rand,
    Draw,
    BranchKeyDown,
    BranchKeyUp,
    GetTimer,
    SetTimer,
    SetSound,
    GetCharAddr,
    StoreBCD,
    Store,
    Load,
    Register(u8),
    Value(u16),
    Comment,
    Comma,
}

fn parse_hex(input: &str) -> Result<u16, LexError> {
    let mut num = 0;
    for c in input.chars() {
        if c >= '0' && c <= '9' {
            num = (num << 4) | (c as u16 - '0' as u16);
        } else if c >= 'a' && c <= 'f' {
            num = (num << 4) | (c as u16 - 'a' as u16 + 10);
        } else if c >= 'A' && c <= 'F' {
            num = (num << 4) | (c as u16 - 'A' as u16 + 10);
        }
    }
    Ok(num)
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LexError {
    IllegalToken,
    NumberTooWide,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ParseError {
    UnknownInstruction,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CompileError {
    LexError(LexError),
    ParseError(ParseError),
}

fn lex(source: &str) -> Result<Vec<Token>, LexError> {
    let r_clr = Regex::new(r"^clr").unwrap();
    let r_ret = Regex::new(r"^ret").unwrap();
    let r_j = Regex::new(r"^j").unwrap();
    let r_call = Regex::new(r"^call").unwrap();
    let r_be = Regex::new(r"^be").unwrap();
    let r_bne = Regex::new(r"^bne").unwrap();
    let r_mov = Regex::new(r"^mov").unwrap();
    let r_add = Regex::new(r"^add").unwrap();
    let r_sub = Regex::new(r"^sub").unwrap();
    let r_and = Regex::new(r"^and").unwrap();
    let r_or = Regex::new(r"^or").unwrap();
    let r_xor = Regex::new(r"^xor").unwrap();
    let r_sr = Regex::new(r"^sr").unwrap();
    let r_sl = Regex::new(r"^sl").unwrap();
    let r_jr = Regex::new(r"^jr").unwrap();
    let r_rand = Regex::new(r"^rand").unwrap();
    let r_draw = Regex::new(r"^draw").unwrap();
    let r_bkd = Regex::new(r"^bkd").unwrap();
    let r_bku = Regex::new(r"^bku").unwrap();
    let r_gdt = Regex::new(r"^gdt").unwrap();
    let r_sdt = Regex::new(r"^sdt").unwrap();
    let r_sst = Regex::new(r"^sst").unwrap();
    let r_gca = Regex::new(r"^gca").unwrap();
    let r_sbcd = Regex::new(r"^sbcd").unwrap();
    let r_sb = Regex::new(r"^sb").unwrap();
    let r_lb = Regex::new(r"^lb").unwrap();
    let r_reg = Regex::new(r"^(V|v)((?<dec>[0-9]{2})|(?<hex>[0-9a-fA-F]))").unwrap();
    let r_val = Regex::new(r"^(?<hex>0x[0-9a-fA-F]+)|^(?<dec>[0-9]+)").unwrap();
    let r_comment = Regex::new(r"^#.?").unwrap();
    let r_comma = Regex::new(r"^,").unwrap();
    let r_whitespace = Regex::new(r"^\s+").unwrap();

    let mut tokens: Vec<Token> = Vec::new();

    'nextline: for (line_number, mut line) in source.lines().enumerate() {
        while !line.is_empty() {
            if r_val.is_match(line) {
                let caps = r_val.captures(line).unwrap();
                let val: u16;
                if let Some(dec) = caps.name("dec") {
                    match dec.as_str().parse::<u16>() {
                        Ok(num) => val = num,
                        Err(_) => return Err(LexError::NumberTooWide),
                    }
                    tokens.push(Token::Value(val));
                    line = &line[dec.len()..];
                } else if let Some(hex) = caps.name("hex") {
                    val = parse_hex(&hex.as_str()[2..])?;
                    tokens.push(Token::Value(val));
                    line = &line[hex.len()..];
                } else {
                    return Err(LexError::IllegalToken);
                }
            } else if r_reg.is_match(line) {
                let caps = r_reg.captures(line).unwrap();
                let val: u8;
                if let Some(dec) = caps.name("dec") {
                    match dec.as_str().parse::<u8>() {
                        Ok(num) => val = num,
                        Err(_) => return Err(LexError::NumberTooWide),
                    }
                    tokens.push(Token::Register(val));
                    line = &line[(dec.len() + 1)..];
                } else if let Some(hex) = caps.name("hex") {
                    val = parse_hex(&hex.as_str())? as u8;
                    tokens.push(Token::Register(val));
                    line = &line[(hex.len() + 1)..];
                } else {
                    return Err(LexError::IllegalToken);
                }
            } else if r_clr.is_match(line) {
                tokens.push(Token::Clear);
                line = &line[3..];
            } else if r_ret.is_match(line) {
                tokens.push(Token::Return);
                line = &line[3..];
            } else if r_j.is_match(line) {
                tokens.push(Token::Jump);
                line = &line[1..];
            } else if r_call.is_match(line) {
                tokens.push(Token::Call);
                line = &line[4..];
            } else if r_be.is_match(line) {
                tokens.push(Token::BranchEqual);
                line = &line[2..];
            } else if r_bne.is_match(line) {
                tokens.push(Token::BranchNotEqual);
                line = &line[3..];
            } else if r_mov.is_match(line) {
                tokens.push(Token::Move);
                line = &line[3..];
            } else if r_add.is_match(line) {
                tokens.push(Token::Add);
                line = &line[3..];
            } else if r_sub.is_match(line) {
                tokens.push(Token::Sub);
                line = &line[3..];
            } else if r_and.is_match(line) {
                tokens.push(Token::And);
                line = &line[3..];
            } else if r_or.is_match(line) {
                tokens.push(Token::Xor);
                line = &line[2..];
            } else if r_xor.is_match(line) {
                tokens.push(Token::Sub);
                line = &line[3..];
            } else if r_sr.is_match(line) {
                tokens.push(Token::ShiftRight);
                line = &line[2..];
            } else if r_sl.is_match(line) {
                tokens.push(Token::ShiftLeft);
                line = &line[2..];
            } else if r_jr.is_match(line) {
                tokens.push(Token::JumpReg);
                line = &line[2..];
            } else if r_rand.is_match(line) {
                tokens.push(Token::Rand);
                line = &line[4..];
            } else if r_draw.is_match(line) {
                tokens.push(Token::Draw);
                line = &line[4..];
            } else if r_bkd.is_match(line) {
                tokens.push(Token::BranchKeyDown);
                line = &line[3..];
            } else if r_bku.is_match(line) {
                tokens.push(Token::BranchKeyUp);
                line = &line[3..];
            } else if r_gdt.is_match(line) {
                tokens.push(Token::GetTimer);
                line = &line[3..];
            } else if r_sdt.is_match(line) {
                tokens.push(Token::SetTimer);
                line = &line[3..];
            } else if r_sst.is_match(line) {
                tokens.push(Token::SetSound);
                line = &line[3..];
            } else if r_gca.is_match(line) {
                tokens.push(Token::GetCharAddr);
                line = &line[3..];
            } else if r_sbcd.is_match(line) {
                tokens.push(Token::StoreBCD);
                line = &line[4..];
            } else if r_sb.is_match(line) {
                tokens.push(Token::Store);
                line = &line[2..];
            } else if r_lb.is_match(line) {
                tokens.push(Token::Load);
                line = &line[2..];
            } else if r_comma.is_match(line) {
                tokens.push(Token::Comma);
                line = &line[1..];
            } else if r_comment.is_match(line) {
                continue 'nextline;
            } else if r_whitespace.is_match(line) {
                // make skip by number of ws characters found
                line = &line[1..];
            } else {
                return Err(LexError::IllegalToken);
            }
        }
    }

    Ok(tokens)
}

fn parse(ast: &[Token]) -> Result<Vec<Instr>, ParseError> {
    let instr_list: Vec<Instr> = Vec::new();
    let mut ast = ast.iter().peekable();

    while ast.peek().is_some() {
        match ast.next() {
            _ => {}
        }
    }

    Ok(instr_list)
}

fn compile(isa: &[Instr]) -> Result<Vec<u8>, CompileError> {
    let bin: Vec<u8> = Vec::new();
    Ok(bin)
}

pub fn assemble(source: &str) -> Result<Vec<u8>, CompileError> {
    let mut bin: Vec<u8> = Vec::new();

    let ast = match lex(source) {
        Ok(v) => v,
        Err(e) => return Err(CompileError::LexError(e)),
    };
    println!("ast: {:?}", ast);

    let isa = match parse(&ast) {
        Ok(v) => v,
        Err(e) => return Err(CompileError::ParseError(e)),
    };
    println!("isa: {:?}", isa);

    compile(&isa)
}
