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
    JumpLabel(String),
    JumpRegLabel(String),
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    SysCall,
    Clear,
    Return,
    Jump,
    Call,
    BranchEqual,
    BranchNotEqual,
    Move,
    MoveI,
    Add,
    Or,
    And,
    AddI,
    Xor,
    Sub,
    SubRegNeg,
    ShiftRight,
    ShiftLeft,
    JumpReg,
    Rand,
    Draw,
    BranchKeyDown,
    BranchKeyUp,
    GetKey,
    GetTimer,
    SetTimer,
    SetSound,
    GetCharAddr,
    StoreBCD,
    Store,
    Load,
    Register(u8),
    Value(u16),
    Label(String),
    Symbol(String),
    //Comment,
    Comma,
    EndLine,
}

#[derive(Debug)]
struct Prog {
    instructions: Vec<Instr>,
    label_map: Vec<(String, u16)>,
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
    MalformedInstruction,
    UndefinedSymbol,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CompileError {
    LexError(LexError),
    ParseError(ParseError),
}

fn lex(source: &str) -> Result<Vec<Token>, LexError> {
    let r_sys = Regex::new(r"^sys\s").unwrap();
    let r_clr = Regex::new(r"^clr\s").unwrap();
    let r_ret = Regex::new(r"^ret\s").unwrap();
    let r_j = Regex::new(r"^j\s").unwrap();
    let r_call = Regex::new(r"^call\s").unwrap();
    let r_be = Regex::new(r"^be\s").unwrap();
    let r_bne = Regex::new(r"^bne\s").unwrap();
    let r_mov = Regex::new(r"^mov\s").unwrap();
    let r_movi = Regex::new(r"^movi\s").unwrap();
    let r_add = Regex::new(r"^add\s").unwrap();
    let r_addi = Regex::new(r"^addi\s").unwrap();
    let r_sub = Regex::new(r"^sub\s").unwrap();
    let r_subn = Regex::new(r"^subn\s").unwrap();
    let r_and = Regex::new(r"^and\s").unwrap();
    let r_or = Regex::new(r"^or\s").unwrap();
    let r_xor = Regex::new(r"^xor\s").unwrap();
    let r_sr = Regex::new(r"^sr\s").unwrap();
    let r_sl = Regex::new(r"^sl\s").unwrap();
    let r_jr = Regex::new(r"^jr\s").unwrap();
    let r_rand = Regex::new(r"^rand\s").unwrap();
    let r_draw = Regex::new(r"^draw\s").unwrap();
    let r_bkd = Regex::new(r"^bkd\s").unwrap();
    let r_bku = Regex::new(r"^bku\s").unwrap();
    let r_gkd = Regex::new(r"^gkd\s").unwrap();
    let r_gdt = Regex::new(r"^gdt\s").unwrap();
    let r_sdt = Regex::new(r"^sdt\s").unwrap();
    let r_sst = Regex::new(r"^sst\s").unwrap();
    let r_gca = Regex::new(r"^gca\s").unwrap();
    let r_sbcd = Regex::new(r"^sbcd\s").unwrap();
    let r_sb = Regex::new(r"^sb\s").unwrap();
    let r_lb = Regex::new(r"^lb\s").unwrap();
    let r_reg = Regex::new(r"^(V|v)((?<dec>[0-9]{2})|(?<hex>[0-9a-fA-F]))").unwrap();
    let r_val = Regex::new(r"^(?<hex>0x[0-9a-fA-F]+)|^(?<dec>[0-9]+)").unwrap();
    let r_comment = Regex::new(r"^#.?").unwrap();
    let r_label_def = Regex::new(r"^(?<name>[a-zA-Z][a-zA-Z0-9]*):").unwrap();
    let r_symbol = Regex::new(r"^(?<name>[a-zA-Z][a-zA-Z0-9]*)").unwrap();
    let r_comma = Regex::new(r"^,").unwrap();
    let r_whitespace = Regex::new(r"^\s+").unwrap();

    let mut tokens: Vec<Token> = Vec::new();

    for (_line_number, mut line) in source.lines().enumerate() {
        while !line.is_empty() {
            if r_val.is_match(line) {
                let caps = r_val.captures(line).unwrap();
                let val: u16;
                if let Some(dec) = caps.name("dec") {
                    match dec.as_str().parse::<u16>() {
                        Ok(num) => {
                            if num & 0xF000 != 0x0000 {
                                return Err(LexError::NumberTooWide);
                            }
                            val = num;
                        }
                        Err(_) => return Err(LexError::NumberTooWide),
                    }
                    tokens.push(Token::Value(val));
                    line = &line[dec.len()..];
                } else if let Some(hex) = caps.name("hex") {
                    val = parse_hex(&hex.as_str()[2..])?;
                    if val & 0xF000 != 0x0000 {
                        return Err(LexError::NumberTooWide);
                    }

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
                        Ok(num) => {
                            if num & 0xF0 != 0x00 {
                                return Err(LexError::NumberTooWide);
                            }
                            val = num;
                        }
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
            } else if r_rand.is_match(line) {
                tokens.push(Token::Rand);
                line = &line[5..];
            } else if r_draw.is_match(line) {
                tokens.push(Token::Draw);
                line = &line[5..];
            } else if r_sbcd.is_match(line) {
                tokens.push(Token::StoreBCD);
                line = &line[5..];
            } else if r_call.is_match(line) {
                tokens.push(Token::Call);
                line = &line[5..];
            } else if r_subn.is_match(line) {
                tokens.push(Token::SubRegNeg);
                line = &line[5..];
            } else if r_sys.is_match(line) {
                tokens.push(Token::SysCall);
                line = &line[4..];
            } else if r_bkd.is_match(line) {
                tokens.push(Token::BranchKeyDown);
                line = &line[4..];
            } else if r_bku.is_match(line) {
                tokens.push(Token::BranchKeyUp);
                line = &line[4..];
            } else if r_gkd.is_match(line) {
                tokens.push(Token::GetKey);
                line = &line[4..]
            } else if r_gdt.is_match(line) {
                tokens.push(Token::GetTimer);
                line = &line[4..];
            } else if r_sdt.is_match(line) {
                tokens.push(Token::SetTimer);
                line = &line[4..];
            } else if r_sst.is_match(line) {
                tokens.push(Token::SetSound);
                line = &line[4..];
            } else if r_gca.is_match(line) {
                tokens.push(Token::GetCharAddr);
                line = &line[4..];
            } else if r_clr.is_match(line) {
                tokens.push(Token::Clear);
                line = &line[4..];
            } else if r_ret.is_match(line) {
                tokens.push(Token::Return);
                line = &line[4..];
            } else if r_bne.is_match(line) {
                tokens.push(Token::BranchNotEqual);
                line = &line[4..];
            } else if r_mov.is_match(line) {
                tokens.push(Token::Move);
                line = &line[4..];
            } else if r_movi.is_match(line) {
                tokens.push(Token::MoveI);
                line = &line[4..];
            } else if r_addi.is_match(line) {
                tokens.push(Token::AddI);
                line = &line[4..];
            } else if r_add.is_match(line) {
                tokens.push(Token::Add);
                line = &line[4..];
            } else if r_sub.is_match(line) {
                tokens.push(Token::Sub);
                line = &line[4..];
            } else if r_xor.is_match(line) {
                tokens.push(Token::Xor);
                line = &line[4..];
            } else if r_and.is_match(line) {
                tokens.push(Token::And);
                line = &line[4..];
            } else if r_or.is_match(line) {
                tokens.push(Token::Or);
                line = &line[3..];
            } else if r_be.is_match(line) {
                tokens.push(Token::BranchEqual);
                line = &line[3..];
            } else if r_sr.is_match(line) {
                tokens.push(Token::ShiftRight);
                line = &line[3..];
            } else if r_sl.is_match(line) {
                tokens.push(Token::ShiftLeft);
                line = &line[3..];
            } else if r_jr.is_match(line) {
                tokens.push(Token::JumpReg);
                line = &line[3..];
            } else if r_sb.is_match(line) {
                tokens.push(Token::Store);
                line = &line[3..];
            } else if r_lb.is_match(line) {
                tokens.push(Token::Load);
                line = &line[3..];
            } else if r_j.is_match(line) {
                tokens.push(Token::Jump);
                line = &line[2..];
            } else if r_comma.is_match(line) {
                tokens.push(Token::Comma);
                line = &line[2..];
            } else if r_comment.is_match(line) {
                break;
            } else if r_label_def.is_match(line) {
                let name: &str = &r_label_def.captures(line).unwrap()["name"];
                tokens.push(Token::Label(name.to_string()));
                line = &line[(name.len() + 1)..];
            } else if r_symbol.is_match(line) {
                let name: &str = &r_symbol.captures(line).unwrap()["name"];
                tokens.push(Token::Symbol(name.to_string()));
                line = &line[name.len()..];
            } else if r_whitespace.is_match(line) {
                // make skip by number of ws characters found
                line = &line[(r_whitespace.captures(line).unwrap()[0].len())..];
            } else {
                return Err(LexError::IllegalToken);
            }
        }
        tokens.push(Token::EndLine);
    }

    Ok(tokens)
}

fn parse(mut ast: &[Token]) -> Result<Prog, ParseError> {
    let mut instr_list: Vec<Instr> = Vec::new();
    let mut label_map: Vec<(String, u16)> = Vec::new();
    let mut symbol_list: Vec<String> = Vec::new();
    let mut address: u16 = 0;

    while !ast.is_empty() {
        match ast {
            [Token::SysCall, Token::Value(v), Token::EndLine, ..] => {
                instr_list.push(Instr::CallMachineCode(*v));
                ast = &ast[3..];
                address += 1;
            }
            [Token::Clear, Token::EndLine, ..] => {
                instr_list.push(Instr::ClearScreen);
                ast = &ast[2..];
                address += 1;
            }
            [Token::Return, Token::EndLine, ..] => {
                instr_list.push(Instr::Return);
                ast = &ast[2..];
                address += 1;
            }
            [Token::Jump, Token::Value(v), Token::EndLine, ..] => {
                instr_list.push(Instr::Jump(*v));
                ast = &ast[3..];
                address += 1;
            }
            [Token::JumpReg, Token::Value(v), Token::EndLine, ..] => {
                instr_list.push(Instr::JumpReg(*v));
                ast = &ast[3..];
                address += 1;
            }
            [Token::Jump, Token::Symbol(s), Token::EndLine, ..] => {
                instr_list.push(Instr::JumpLabel(s.clone()));
                symbol_list.push(s.clone());
                ast = &ast[3..];
                address += 1;
            }
            [Token::JumpReg, Token::Symbol(s), Token::EndLine, ..] => {
                instr_list.push(Instr::JumpRegLabel(s.clone()));
                symbol_list.push(s.clone());
                ast = &ast[3..];
                address += 1;
            }
            [Token::Call, Token::Value(v), Token::EndLine, ..] => {
                instr_list.push(Instr::Call(*v));
                ast = &ast[3..];
                address += 1;
            }
            [Token::BranchEqual, Token::Register(r1), Token::Comma, Token::Value(v), Token::EndLine, ..] =>
            {
                instr_list.push(Instr::IfEqualImm(*r1, *v as u8));
                ast = &ast[5..];
                address += 1;
            }
            [Token::BranchEqual, Token::Register(r1), Token::Comma, Token::Register(r2), Token::EndLine, ..] =>
            {
                instr_list.push(Instr::IfEqualReg(*r1, *r2 as u8));
                ast = &ast[5..];
                address += 1;
            }
            [Token::BranchNotEqual, Token::Register(r1), Token::Comma, Token::Value(v), Token::EndLine, ..] =>
            {
                instr_list.push(Instr::IfNotEqualImm(*r1, *v as u8));
                ast = &ast[5..];
                address += 1;
            }
            [Token::BranchNotEqual, Token::Register(r1), Token::Comma, Token::Register(r2), Token::EndLine, ..] =>
            {
                instr_list.push(Instr::IfNotEqualReg(*r1, *r2 as u8));
                ast = &ast[5..];
                address += 1;
            }
            [Token::Move, Token::Register(r1), Token::Comma, Token::Value(v), Token::EndLine, ..] =>
            {
                instr_list.push(Instr::SetImm(*r1, *v as u8));
                ast = &ast[5..];
                address += 1;
            }
            [Token::Move, Token::Register(r1), Token::Comma, Token::Register(r2), Token::EndLine, ..] =>
            {
                instr_list.push(Instr::SetReg(*r1, *r2 as u8));
                ast = &ast[5..];
                address += 1;
            }
            [Token::MoveI, Token::Value(v), Token::EndLine, ..] => {
                instr_list.push(Instr::SetI(*v));
                ast = &ast[3..];
                address += 1;
            }
            [Token::AddI, Token::Register(r1), Token::EndLine, ..] => {
                instr_list.push(Instr::AddIReg(*r1));
                ast = &ast[3..];
                address += 1;
            }
            [Token::Add, Token::Register(r1), Token::Comma, Token::Value(v), Token::EndLine, ..] => {
                instr_list.push(Instr::AddImm(*r1, *v as u8));
                ast = &ast[5..];
                address += 1;
            }
            [Token::Add, Token::Register(r1), Token::Comma, Token::Register(r2), Token::EndLine, ..] =>
            {
                instr_list.push(Instr::AddReg(*r1, *r2 as u8));
                ast = &ast[5..];
                address += 1;
            }
            [Token::Sub, Token::Register(r1), Token::Comma, Token::Register(r2), Token::EndLine, ..] =>
            {
                instr_list.push(Instr::SubReg(*r1, *r2 as u8));
                ast = &ast[5..];
                address += 1;
            }
            [Token::SubRegNeg, Token::Register(r1), Token::Comma, Token::Register(r2), Token::EndLine, ..] =>
            {
                instr_list.push(Instr::SetSubReg(*r1, *r2 as u8));
                ast = &ast[5..];
                address += 1;
            }
            [Token::Or, Token::Register(r1), Token::Comma, Token::Register(r2), Token::EndLine, ..] =>
            {
                instr_list.push(Instr::OrReg(*r1, *r2 as u8));
                ast = &ast[5..];
                address += 1;
            }
            [Token::And, Token::Register(r1), Token::Comma, Token::Register(r2), Token::EndLine, ..] =>
            {
                instr_list.push(Instr::AndReg(*r1, *r2 as u8));
                ast = &ast[5..];
                address += 1;
            }
            [Token::Xor, Token::Register(r1), Token::Comma, Token::Register(r2), Token::EndLine, ..] =>
            {
                instr_list.push(Instr::XorReg(*r1, *r2 as u8));
                ast = &ast[5..];
                address += 1;
            }
            [Token::ShiftLeft, Token::Register(r1), Token::Comma, Token::Value(v), Token::EndLine, ..] =>
            {
                instr_list.push(Instr::ShiftLeft(*r1, *v as u8));
                ast = &ast[5..];
                address += 1;
            }
            [Token::ShiftRight, Token::Register(r1), Token::Comma, Token::Value(v), Token::EndLine, ..] =>
            {
                instr_list.push(Instr::ShiftRight(*r1, *v as u8));
                ast = &ast[5..];
                address += 1;
            }
            [Token::Rand, Token::Register(r1), Token::Comma, Token::Value(v), Token::EndLine, ..] =>
            {
                instr_list.push(Instr::Rand(*r1, *v as u8));
                ast = &ast[5..];
                address += 1;
            }
            [Token::Draw, Token::Register(r1), Token::Comma, Token::Register(r2), Token::Comma, Token::Value(v), Token::EndLine, ..] =>
            {
                instr_list.push(Instr::Draw(*r1, *r2, *v as u8));
                ast = &ast[7..];
                address += 1;
            }
            [Token::BranchKeyUp, Token::Register(r1), Token::EndLine, ..] => {
                instr_list.push(Instr::IfKey(*r1));
                ast = &ast[3..];
                address += 1;
            }
            [Token::BranchKeyDown, Token::Register(r1), Token::EndLine, ..] => {
                instr_list.push(Instr::IfNotKey(*r1));
                ast = &ast[3..];
                address += 1;
            }

            [Token::GetTimer, Token::Register(r1), Token::EndLine, ..] => {
                instr_list.push(Instr::GetTimer(*r1));
                ast = &ast[3..];
                address += 1;
            }
            [Token::SetTimer, Token::Register(r1), Token::EndLine, ..] => {
                instr_list.push(Instr::SetTimer(*r1));
                ast = &ast[3..];
                address += 1;
            }
            [Token::SetSound, Token::Register(r1), Token::EndLine, ..] => {
                instr_list.push(Instr::SetSound(*r1));
                ast = &ast[3..];
                address += 1;
            }
            [Token::GetCharAddr, Token::Register(r1), Token::EndLine, ..] => {
                instr_list.push(Instr::SetICharAddr(*r1));
                ast = &ast[3..];
                address += 1;
            }
            [Token::StoreBCD, Token::Register(r1), Token::EndLine, ..] => {
                instr_list.push(Instr::StoreDecimal(*r1));
                ast = &ast[3..];
                address += 1;
            }
            [Token::Store, Token::Register(r1), Token::EndLine, ..] => {
                instr_list.push(Instr::StoreReg(*r1));
                ast = &ast[3..];
                address += 1;
            }
            [Token::Load, Token::Register(r1), Token::EndLine, ..] => {
                instr_list.push(Instr::LoadReg(*r1));
                ast = &ast[3..];
                address += 1;
            }
            [Token::GetKey, Token::Register(r1), Token::EndLine, ..] => {
                instr_list.push(Instr::GetKey(*r1));
                ast = &ast[3..];
                address += 1;
            }
            [Token::EndLine, ..] => ast = &ast[1..],
            [Token::Label(s), ..] => {
                label_map.push((s.clone(), address));
                ast = &ast[1..];
            }
            _ => {
                //println!("{:?}", ast[0]);
                return Err(ParseError::MalformedInstruction);
            }
        }
    }

    for s in symbol_list {
        if label_map.iter().find(|&l| l.0 == *s).is_none() {
            return Err(ParseError::UndefinedSymbol);
        }
    }

    Ok(Prog {
        instructions: instr_list,
        label_map: label_map,
    })
}

fn compile(isa: &Prog) -> Vec<u8> {
    let bin: Vec<u8> = isa
        .instructions
        .iter()
        .map(|instr| match instr {
            Instr::CallMachineCode(v) => [(v >> 8) as u8 & 0x0F, (v & 0x00FF) as u8],
            Instr::ClearScreen => [0x00, 0xE0],
            Instr::Return => [0x00, 0xEE],
            Instr::Jump(v) => [0x10 | (v >> 8) as u8 & 0x0F, (v & 0x00FF) as u8],
            Instr::Call(v) => [0x20 | (v >> 8) as u8 & 0x0F, (v & 0x00FF) as u8],
            Instr::IfEqualImm(x, v) => [0x30 | (x & 0x0F), *v as u8],
            Instr::IfNotEqualImm(x, v) => [0x40 | (x & 0x0F), *v as u8],
            Instr::IfEqualReg(x, y) => [0x50 | (x & 0x0F), y << 4],
            Instr::SetImm(x, v) => [0x60 | (x & 0x0F), *v as u8],
            Instr::AddImm(x, v) => [0x70 | (x & 0x0F), *v as u8],
            Instr::SetReg(x, y) => [0x80 | (x & 0x0F), (y << 4) | 0x00],
            Instr::OrReg(x, y) => [0x80 | (x & 0x0F), (y << 4) | 0x01],
            Instr::AndReg(x, y) => [0x80 | (x & 0x0F), (y << 4) | 0x02],
            Instr::XorReg(x, y) => [0x80 | (x & 0x0F), (y << 4) | 0x03],
            Instr::AddReg(x, y) => [0x80 | (x & 0x0F), (y << 4) | 0x04],
            Instr::SubReg(x, y) => [0x80 | (x & 0x0F), (y << 4) | 0x05],
            Instr::ShiftRight(x, y) => [0x80 | (x & 0x0F), (y << 4) | 0x06],
            Instr::SetSubReg(x, y) => [0x80 | (x & 0x0F), (y << 4) | 0x07],
            Instr::ShiftLeft(x, y) => [0x80 | (x & 0x0F), (y << 4) | 0x0E],
            Instr::IfNotEqualReg(x, y) => [0x90 | (x & 0x0F), y << 4],
            Instr::SetI(v) => [0xA0 | (v >> 8) as u8 & 0x0F, (v & 0x00FF) as u8],
            Instr::JumpReg(v) => [0xB0 | (v >> 8) as u8 & 0x0F, (v & 0x00FF) as u8],
            Instr::Rand(x, v) => [0xC0 | (x & 0x0F), *v as u8],
            Instr::Draw(x, y, v) => [0xD0 | (x & 0x0F), (y << 4) | (v & 0x000F) as u8],
            Instr::IfKey(x) => [0xE | (x & 0x0F), 0x9E],
            Instr::IfNotKey(x) => [0xE | (x & 0x0F), 0xA1],
            Instr::GetTimer(x) => [0xF0 | (x & 0x0F), 0x07],
            Instr::GetKey(x) => [0xF0 | (x & 0x0F), 0x0A],
            Instr::SetTimer(x) => [0xF0 | (x & 0x0F), 0x15],
            Instr::SetSound(x) => [0xF0 | (x & 0x0F), 0x18],
            Instr::AddIReg(x) => [0xF0 | (x & 0x0F), 0x1E],
            Instr::SetICharAddr(x) => [0xF0 | (x & 0x0F), 0x29],
            Instr::StoreDecimal(x) => [0xF0 | (x & 0x0F), 0x33],
            Instr::StoreReg(x) => [0xF0 | (x & 0x0F), 0x55],
            Instr::LoadReg(x) => [0xF0 | (x & 0x0F), 0x65],
            Instr::JumpLabel(s) => {
                let target = isa.label_map.iter().find(|&x| x.0 == *s).unwrap().1;
                [0x10 | (target >> 8) as u8 & 0x0F, (target & 0x00FF) as u8]
            }
            Instr::JumpRegLabel(s) => {
                let target = isa.label_map.iter().find(|&x| x.0 == *s).unwrap().1;
                [0xB0 | (target >> 8) as u8 & 0x0F, (target & 0x00FF) as u8]
            }
        })
        .flatten()
        .collect();

    bin
}

pub fn assemble(source: &str, print_debug: bool) -> Result<Vec<u8>, CompileError> {
    let ast = match lex(source) {
        Ok(v) => v,
        Err(e) => return Err(CompileError::LexError(e)),
    };
    if print_debug {
        println!("ast: {:?}", ast);
    }

    let isa = match parse(&ast) {
        Ok(v) => v,
        Err(e) => return Err(CompileError::ParseError(e)),
    };
    if print_debug {
        println!("isa: {:?}", isa);
    }

    Ok(compile(&isa))
}
