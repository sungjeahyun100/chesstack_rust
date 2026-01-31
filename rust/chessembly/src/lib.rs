#![allow(dead_code)]

use std::collections::HashMap;

/// 디버그 로그 출력 (WASM 환경에서는 JS console.log로 전달)
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[cfg(target_arch = "wasm32")]
fn log_debug(msg: &str) {
    unsafe {
        log(msg);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn log_debug(msg: &str) {
    println!("DEBUG: {}", msg);
}

/// 행마법 종류
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveType {
    TakeMove, // 이동 또는 잡기
    Move,     // 이동만 (빈 칸만)
    Take,     // 잡기만 (적 있을 때만)
    Catch,    // 제자리에서 잡기 (원거리 공격)
    Shift,    // 자리 바꾸기
    Jump,     // take 후 점프
}

/// 액션 태그 종류
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionTagType {
    Transition, // 기물 변환
    SetState,   // 상태 설정
}

/// 활성화된 칸에 부착되는 액션 태그
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionTag {
    pub tag_type: ActionTagType,
    pub key: String,
    pub value: i32,
    pub piece_name: Option<String>,
}

/// 활성화된 칸 정보
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activation {
    pub dx: i32,              // 기물 위치 기준 x 오프셋
    pub dy: i32,              // 기물 위치 기준 y 오프셋
    pub move_type: MoveType,  // 행마법 종류
    pub tags: Vec<ActionTag>, // 부착된 액션 태그들
    pub catch_to: Option<(i32, i32)>, //jump행마용 기물 잡는 곳 저장소
}

/// 보드 상태 (외부에서 제공)
pub struct BoardState {
    pub board_width: i32,
    pub board_height: i32,
    pub piece_x: i32,
    pub piece_y: i32,
    pub piece_name: String,
    pub is_white: bool,
    /// (x, y) -> (piece_name, is_white)
    pub pieces: HashMap<(i32, i32), (String, bool)>,
    /// 전역 상태
    pub state: HashMap<String, i32>,
    /// 위협받는 칸들 (적에게 공격받는 위치)
    pub danger_squares: std::collections::HashSet<(i32, i32)>,
    /// 현재 체크 상태인지
    pub in_check: bool,
}

impl BoardState {
    /// 해당 좌표가 보드 안인지
    fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.board_width && y >= 0 && y < self.board_height
    }
    
    /// 해당 좌표가 비어있는지
    fn is_empty(&self, x: i32, y: i32) -> bool {
        self.in_bounds(x, y) && !self.pieces.contains_key(&(x, y))
    }
    
    /// 해당 좌표에 적이 있는지
    fn has_enemy(&self, x: i32, y: i32) -> bool {
        if let Some((_, is_white)) = self.pieces.get(&(x, y)) {
            *is_white != self.is_white
        } else {
            false
        }
    }
    
    /// 해당 좌표에 아군이 있는지
    fn has_friendly(&self, x: i32, y: i32) -> bool {
        if let Some((_, is_white)) = self.pieces.get(&(x, y)) {
            *is_white == self.is_white
        } else {
            false
        }
    }
    
    /// 해당 좌표에 특정 기물이 있는지
    fn has_piece(&self, x: i32, y: i32, piece_name: &str) -> bool {
        if let Some((name, _)) = self.pieces.get(&(x, y)) {
            name == piece_name
        } else {
            false
        }
    }
}

/// 토큰 종류
#[derive(Debug, Clone, PartialEq)]
enum Token {
    // 행마식
    TakeMove(i32, i32),
    Move(i32, i32),
    Take(i32, i32),
    Catch(i32, i32),
    Shift(i32, i32),
    Jump(i32, i32),
    Anchor(i32, i32),
    
    // 조건식
    Observe(i32, i32),
    Peek(i32, i32),
    Enemy(i32, i32),
    Friendly(i32, i32),
    PieceOn(String, i32, i32),
    Danger(i32, i32),
    Check,
    Bound(i32, i32),
    Edge(i32, i32),
    EdgeTop(i32, i32),
    EdgeBottom(i32, i32),
    EdgeLeft(i32, i32),
    EdgeRight(i32, i32),
    Corner(i32, i32),
    CornerTopLeft(i32, i32),
    CornerTopRight(i32, i32),
    CornerBottomLeft(i32, i32),
    CornerBottomRight(i32, i32),
    
    // 상태 관련
    Piece(String),
    IfState(String, i32),
    SetState(String, i32),
    SetStateReset,
    Transition(String),
    
    // 제어
    Repeat(usize),
    Do,
    While,
    Jmp(String),
    Jne(String),
    Label(String),
    Not,
    End,
    
    // 구조
    OpenBrace,
    CloseBrace,
    Semicolon,
}

/// 렉서
struct Lexer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }
    
    fn skip_whitespace(&mut self) {
        let bytes = self.input.as_bytes();
        while self.pos < bytes.len() && (bytes[self.pos] as char).is_whitespace() {
            self.pos += 1;
        }
    }
    
    fn skip_comment(&mut self) {
        let bytes = self.input.as_bytes();
        if self.pos < bytes.len() && bytes[self.pos] == b'#' {
            while self.pos < bytes.len() && bytes[self.pos] != b'\n' {
                self.pos += 1;
            }
        }
    }
    
    fn read_word(&mut self) -> String {
        let bytes = self.input.as_bytes();
        let start = self.pos;
        while self.pos < bytes.len() {
            let ch = bytes[self.pos] as char;
            if ch.is_whitespace() || ";{}(),#".contains(ch) {
                break;
            }
            self.pos += 1;
        }
        self.input[start..self.pos].to_string()
    }
    
    fn read_args(&mut self) -> Vec<String> {
        let mut args = Vec::new();
        self.skip_whitespace();
        let bytes = self.input.as_bytes();
        
        if self.pos >= bytes.len() || bytes[self.pos] != b'(' {
            return args;
        }
        self.pos += 1; // consume '('
        
        let mut current = String::new();
        let mut depth = 0;
        
        while self.pos < bytes.len() {
            let ch = bytes[self.pos] as char;
            self.pos += 1;
            
            match ch {
                '(' => {
                    depth += 1;
                    current.push(ch);
                }
                ')' => {
                    if depth == 0 {
                        let trimmed = current.trim().to_string();
                        if !trimmed.is_empty() {
                            args.push(trimmed);
                        }
                        break;
                    }
                    depth -= 1;
                    current.push(ch);
                }
                ',' if depth == 0 => {
                    let trimmed = current.trim().to_string();
                    if !trimmed.is_empty() {
                        args.push(trimmed);
                    }
                    current.clear();
                }
                _ => current.push(ch),
            }
        }
        
        args
    }
    
    fn next_token(&mut self) -> Option<Token> {
        loop {
            self.skip_whitespace();
            self.skip_comment();
            self.skip_whitespace();
            
            let bytes = self.input.as_bytes();
            if self.pos >= bytes.len() {
                return None;
            }
            
            let ch = bytes[self.pos] as char;
            
            // 단일 문자 토큰
            match ch {
                ';' => { self.pos += 1; return Some(Token::Semicolon); }
                '{' => { self.pos += 1; return Some(Token::OpenBrace); }
                '}' => { self.pos += 1; return Some(Token::CloseBrace); }
                '#' => { self.skip_comment(); continue; }
                _ => {}
            }
            
            // 키워드/식
            let word = self.read_word();
            if word.is_empty() {
                self.pos += 1;
                continue;
            }
            
            let args = self.read_args();
            
            return Some(self.parse_token(&word, args));
        }
    }
    
    fn parse_token(&self, word: &str, args: Vec<String>) -> Token {
        let parse_i32 = |s: &str| s.parse::<i32>().unwrap_or(0);
        let get_xy = |args: &Vec<String>| -> (i32, i32) {
            if args.len() >= 2 {
                (parse_i32(&args[0]), parse_i32(&args[1]))
            } else {
                (0, 0)
            }
        };
        
        match word {
            // 행마식
            "take-move" => { let (dx, dy) = get_xy(&args); Token::TakeMove(dx, dy) }
            "move" => { let (dx, dy) = get_xy(&args); Token::Move(dx, dy) }
            "take" => { let (dx, dy) = get_xy(&args); Token::Take(dx, dy) }
            "catch" => { let (dx, dy) = get_xy(&args); Token::Catch(dx, dy) }
            "shift" => { let (dx, dy) = get_xy(&args); Token::Shift(dx, dy) }
            "jump" => { let (dx, dy) = get_xy(&args); Token::Jump(dx, dy) }
            "anchor" => { let (dx, dy) = get_xy(&args); Token::Anchor(dx, dy) }
            
            // 조건식
            "observe" => { let (dx, dy) = get_xy(&args); Token::Observe(dx, dy) }
            "peek" => { let (dx, dy) = get_xy(&args); Token::Peek(dx, dy) }
            "enemy" => { let (dx, dy) = get_xy(&args); Token::Enemy(dx, dy) }
            "friendly" => { let (dx, dy) = get_xy(&args); Token::Friendly(dx, dy) }
            "piece-on" => {
                if args.len() >= 3 {
                    Token::PieceOn(args[0].clone(), parse_i32(&args[1]), parse_i32(&args[2]))
                } else {
                    Token::End
                }
            }
            "danger" => { let (dx, dy) = get_xy(&args); Token::Danger(dx, dy) }
            "check" => Token::Check,
            "bound" => { let (dx, dy) = get_xy(&args); Token::Bound(dx, dy) }
            "edge" => { let (dx, dy) = get_xy(&args); Token::Edge(dx, dy) }
            "edge-top" => { let (dx, dy) = get_xy(&args); Token::EdgeTop(dx, dy) }
            "edge-bottom" => { let (dx, dy) = get_xy(&args); Token::EdgeBottom(dx, dy) }
            "edge-left" => { let (dx, dy) = get_xy(&args); Token::EdgeLeft(dx, dy) }
            "edge-right" => { let (dx, dy) = get_xy(&args); Token::EdgeRight(dx, dy) }
            "corner" => { let (dx, dy) = get_xy(&args); Token::Corner(dx, dy) }
            "corner-top-left" => { let (dx, dy) = get_xy(&args); Token::CornerTopLeft(dx, dy) }
            "corner-top-right" => { let (dx, dy) = get_xy(&args); Token::CornerTopRight(dx, dy) }
            "corner-bottom-left" => { let (dx, dy) = get_xy(&args); Token::CornerBottomLeft(dx, dy) }
            "corner-bottom-right" => { let (dx, dy) = get_xy(&args); Token::CornerBottomRight(dx, dy) }
            
            // 상태
            "piece" => {
                if args.len() >= 1 {
                    Token::Piece(args[0].clone())
                } else {
                    Token::End
                }
            }
            "if-state" => {
                if args.len() >= 2 {
                    Token::IfState(args[0].clone(), parse_i32(&args[1]))
                } else {
                    Token::End
                }
            }
            "set-state" => {
                if args.len() >= 2 {
                    Token::SetState(args[0].clone(), parse_i32(&args[1]))
                } else {
                    Token::SetStateReset
                }
            }
            "transition" => {
                if args.len() >= 1 {
                    Token::Transition(args[0].clone())
                } else {
                    Token::End
                }
            }
            
            // 제어
            "repeat" => {
                if args.len() >= 1 {
                    Token::Repeat(args[0].parse().unwrap_or(1))
                } else {
                    Token::Repeat(1)
                }
            }
            "do" => Token::Do,
            "while" => Token::While,
            "jmp" => {
                if args.len() >= 1 {
                    Token::Jmp(args[0].clone())
                } else {
                    Token::End
                }
            }
            "jne" => {
                if args.len() >= 1 {
                    Token::Jne(args[0].clone())
                } else {
                    Token::End
                }
            }
            "label" => {
                if args.len() >= 1 {
                    Token::Label(args[0].clone())
                } else {
                    Token::End
                }
            }
            "not" => Token::Not,
            "end" => Token::End,
            
            _ => Token::End, // 알 수 없는 토큰은 end로 처리
        }
    }
}

/// 인터프리터
pub struct Interpreter {
    tokens: Vec<Token>,
    pub debug: bool,  // 디버그 모드 활성화 여부
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            debug: false,
            tokens: Vec::new(),
        }
    }
    
    /// 디버그 모드 설정
    pub fn set_debug(&mut self, enabled: bool) {
        self.debug = enabled;
    }
    
    /// 활성화 추가 (디버그 로깅 포함)
    fn add_activation(&self, activations: &mut Vec<Activation>, activation: Activation) {
        if self.debug {
            log_debug(&format!("    → Activation: ({}, {}) {:?}", 
                activation.dx, activation.dy, activation.move_type));
        }
        activations.push(activation);
    }
    
    /// 스크립트 파싱
    pub fn parse(&mut self, input: &str) {
        let mut lexer = Lexer::new(input);
        self.tokens.clear();
        while let Some(token) = lexer.next_token() {
            self.tokens.push(token);
        }
    }
    
    /// 행마법 계산 실행
    pub fn execute(&self, board: &mut BoardState) -> Vec<Activation> {
        if self.debug {
            log_debug(&format!("[Chessembly] Executing script for {} at ({}, {})", 
                board.piece_name, board.piece_x, board.piece_y));
            log_debug(&format!("[Chessembly] Total tokens: {}", self.tokens.len()));
        }
        
        let mut activations = Vec::new();
        let mut pc = 0usize; // 프로그램 카운터
        // 라벨 인덱스는 실행마다 로컬로 계산하여 체인 종료 시 재설정됩니다.
        let mut labels: HashMap<usize, HashMap<String, usize>> = HashMap::new();

        let mut num_of_open_brace = 0usize; //범위 밖의 닫힌괄호에 인터프리터가 멈추지 않게 하기 위한 카운터

        let mut index_of_expression_chain = 0usize; //몇번째 식 연쇄인지 카운팅 
        
        // 앵커 (기준 위치) - 기물 위치로부터의 누적 오프셋
        let mut anchor_x = 0i32;
        let mut anchor_y = 0i32;
        
        // 실행 상태
        let mut last_value = true;
        
        // 펜딩 액션 태그
        let mut pending_tags: Vec<ActionTag> = Vec::new();
        
        // do...while용 시작 위치
        let mut do_index: Option<usize> = None;
        
        // {} 스코프 스택: (anchor_x, anchor_y, token_index)
        let mut scope_stack: Vec<(i32, i32, usize)> = Vec::new();
        
        // 마지막 take 위치 (jump용)
        let mut last_take_pos: Option<(i32, i32)> = None;

        //label index pre-processing
        while pc < self.tokens.len() {
            let token = &self.tokens[pc];

            pc += 1;

            match token {
                Token::Semicolon => {
                    index_of_expression_chain += 1;
                }
                Token::Label(n) => {
                    labels
                        .entry(index_of_expression_chain)
                        .or_insert_with(HashMap::new)
                        .insert(n.to_string(), pc);
                },
                _ => continue,
            }
        }

        pc = 0usize;
        index_of_expression_chain = 0usize;

        while pc < self.tokens.len() {
            let token = &self.tokens[pc];
            
            if self.debug {
                log_debug(&format!("  [PC:{}] Token: {:?} | Anchor: ({}, {}) | LastValue: {}", 
                    pc, token, anchor_x, anchor_y, last_value));
            }
            
            pc += 1;
            
            // 일반 식이 false를 반환하면 체인 종료 (예외 제외)
            let should_terminate = !last_value && !matches!(token, 
                Token::While | Token::Jmp(_) | Token::Jne(_) | Token::Not | 
                Token::Label(_) | Token::Semicolon | Token::CloseBrace
            );
            
            if should_terminate {
                // 현재 체인(;까지) 스킵
                while pc < self.tokens.len() {
                    match &self.tokens[pc] {
                        Token::Semicolon => { 
                            // 체인 종료: 앵커 초기화
                            anchor_x = 0;
                            anchor_y = 0;
                            pending_tags.clear();
                            do_index = None;
                            last_take_pos = None;
                            pc += 1; 
                            index_of_expression_chain += 1;
                            break; 
                        }
                        Token::CloseBrace => {
                            // 스코프 복원
                            if num_of_open_brace > 0 {
                                num_of_open_brace -= 1;
                                pc += 1;
                                continue;
                            }
                            if let Some((ax, ay, _)) = scope_stack.pop() {
                                anchor_x = ax;
                                anchor_y = ay;
                            }
                            pc += 1;
                            break;
                        }
                        Token::OpenBrace => {
                            num_of_open_brace += 1;
                            pc += 1;
                            continue;
                        }
                        _ => pc += 1,
                    }
                }
                last_value = true;
                continue;
            }
            
            match token {
                Token::Semicolon => {
                    // 체인 종료, 앵커 초기화
                    anchor_x = 0;
                    anchor_y = 0;
                    last_value = true;
                    pending_tags.clear();
                    do_index = None;
                    last_take_pos = None;
                    index_of_expression_chain += 1;
                }
                
                Token::OpenBrace => {
                    // 현재 앵커 저장
                    scope_stack.push((anchor_x, anchor_y, pc));
                    last_value = true;
                }
                
                Token::CloseBrace => {
                    // 앵커 복원
                    if let Some((ax, ay, _)) = scope_stack.pop() {
                        anchor_x = ax;
                        anchor_y = ay;
                    }
                    last_value = true;
                }
                
                // === 행마식 ===
                Token::TakeMove(dx, dy) => {
                    let target_x = board.piece_x + anchor_x + dx;
                    let target_y = board.piece_y + anchor_y + dy;
                    
                    if !board.in_bounds(target_x, target_y) || board.has_friendly(target_x, target_y) {
                        last_value = false;
                    } else if board.has_enemy(target_x, target_y) {
                        self.add_activation(&mut activations, Activation {
                            dx: anchor_x + dx,
                            dy: anchor_y + dy,
                            move_type: MoveType::TakeMove,
                            tags: pending_tags.clone(),
                            catch_to: None,
                        });
                        anchor_x += dx;
                        anchor_y += dy;
                        last_value = false; // 적을 잡으면 체인 종료
                    } else {
                        self.add_activation(&mut activations, Activation {
                            dx: anchor_x + dx,
                            dy: anchor_y + dy,
                            move_type: MoveType::TakeMove,
                            tags: pending_tags.clone(),
                            catch_to: None,
                        });
                        anchor_x += dx;
                        anchor_y += dy;
                        last_value = true;
                    }
                }
                
                Token::Move(dx, dy) => {
                    let target_x = board.piece_x + anchor_x + dx;
                    let target_y = board.piece_y + anchor_y + dy;
                    
                    if board.is_empty(target_x, target_y) {
                        self.add_activation(&mut activations, Activation {
                            dx: anchor_x + dx,
                            dy: anchor_y + dy,
                            move_type: MoveType::Move,
                            tags: pending_tags.clone(),
                            catch_to: None,
                        });
                        anchor_x += dx;
                        anchor_y += dy;
                        last_value = true;
                    } else {
                        last_value = false;
                    }
                }
                
                Token::Take(dx, dy) => {
                    let target_x = board.piece_x + anchor_x + dx;
                    let target_y = board.piece_y + anchor_y + dy;
                    
                    if board.has_enemy(target_x, target_y) {
                        last_take_pos = Some((anchor_x + dx, anchor_y + dy));
                        // take 자체는 jump가 없으면 활성화
                        self.add_activation(&mut activations, Activation {
                            dx: anchor_x + dx,
                            dy: anchor_y + dy,
                            move_type: MoveType::Take,
                            tags: pending_tags.clone(),
                            catch_to: None,
                        });
                        anchor_x += dx;
                        anchor_y += dy;
                        last_value = true;
                    } else {
                        // 적이 없으면 앵커만 이동
                        if board.in_bounds(target_x, target_y) && !board.has_friendly(target_x, target_y) {
                            anchor_x += dx;
                            anchor_y += dy;
                            last_value = true;
                        } else {
                            last_value = false;
                        }
                    }
                }
                
                Token::Jump(dx, dy) => {
                    // 앞의 take가 있고 적이 있었으면 take-jump 활성화
                    if activations.last().unwrap().move_type == MoveType::Take {
                        activations.pop();
                    }
                    if let Some((_take_dx, _take_dy)) = last_take_pos.as_ref() {
                        
                        let target_x = board.piece_x + anchor_x + dx;
                        let target_y = board.piece_y + anchor_y + dy;
                        
                        if board.is_empty(target_x, target_y) {
                            // take 위치를 잡고, jump 위치로 이동하는 행마 활성화
                            self.add_activation(&mut activations, Activation {
                                dx: anchor_x + dx,
                                dy: anchor_y + dy,
                                move_type: MoveType::Jump,
                                tags: pending_tags.clone(),
                                catch_to: last_take_pos,
                            });
                            anchor_x += dx;
                            anchor_y += dy;
                            last_value = true;
                        } else {
                            last_value = false;
                        }
                    } else {
                        last_value = false;
                    }
                }
                
                Token::Catch(dx, dy) => {
                    let target_x = board.piece_x + anchor_x + dx;
                    let target_y = board.piece_y + anchor_y + dy;
                    
                    if board.has_enemy(target_x, target_y) {
                        self.add_activation(&mut activations, Activation {
                            dx: anchor_x + dx,
                            dy: anchor_y + dy,
                            move_type: MoveType::Catch,
                            tags: pending_tags.clone(),
                            catch_to: None,
                        });
                        last_value = true;
                    } else {
                        last_value = false;
                    }
                    // catch는 앵커를 이동하지 않음
                }
                
                Token::Shift(dx, dy) => {
                    let target_x = board.piece_x + anchor_x + dx;
                    let target_y = board.piece_y + anchor_y + dy;
                    
                    if board.in_bounds(target_x, target_y) && !board.is_empty(target_x, target_y) {
                        self.add_activation(&mut activations, Activation {
                            dx: anchor_x + dx,
                            dy: anchor_y + dy,
                            move_type: MoveType::Shift,
                            tags: pending_tags.clone(),
                            catch_to: None,
                        });
                        anchor_x += dx;
                        anchor_y += dy;
                        last_value = true;
                    } else {
                        last_value = false;
                    }
                }
                
                Token::Anchor(dx, dy) => {
                    anchor_x += dx;
                    anchor_y += dy;
                    last_value = true;
                }
                
                // === 조건식 ===
                Token::Observe(dx, dy) => {
                    let target_x = board.piece_x + anchor_x + dx;
                    let target_y = board.piece_y + anchor_y + dy;
                    last_value = board.is_empty(target_x, target_y);
                    // observe는 앵커를 이동하지 않음
                }
                
                Token::Peek(dx, dy) => {
                    let target_x = board.piece_x + anchor_x + dx;
                    let target_y = board.piece_y + anchor_y + dy;
                    if board.is_empty(target_x, target_y) {
                        anchor_x += dx;
                        anchor_y += dy;
                        last_value = true;
                    } else if board.is_empty(target_x, target_y) == false {
                        anchor_x += dx;
                        anchor_y += dy;
                        last_value = false;
                    } else {
                        last_value = false;
                    }
                }
                
                Token::Enemy(dx, dy) => {
                    let target_x = board.piece_x + anchor_x + dx;
                    let target_y = board.piece_y + anchor_y + dy;
                    last_value = board.has_enemy(target_x, target_y);
                }
                
                Token::Friendly(dx, dy) => {
                    let target_x = board.piece_x + anchor_x + dx;
                    let target_y = board.piece_y + anchor_y + dy;
                    last_value = board.has_friendly(target_x, target_y);
                }
                
                Token::PieceOn(name, dx, dy) => {
                    let target_x = board.piece_x + anchor_x + dx;
                    let target_y = board.piece_y + anchor_y + dy;
                    last_value = board.has_piece(target_x, target_y, name);
                }
                
                Token::Danger(dx, dy) => {
                    let target_x = board.piece_x + anchor_x + dx;
                    let target_y = board.piece_y + anchor_y + dy;
                    last_value = board.danger_squares.contains(&(target_x, target_y));
                }
                
                Token::Check => {
                    last_value = board.in_check;
                }
                
                Token::Bound(dx, dy) => {
                    let target_x = board.piece_x + anchor_x + dx;
                    let target_y = board.piece_y + anchor_y + dy;
                    last_value = !board.in_bounds(target_x, target_y);
                }
                
                Token::Edge(dx, dy) => {
                    let target_x = board.piece_x + anchor_x + dx;
                    let target_y = board.piece_y + anchor_y + dy;
                    last_value = target_x < 0 || target_x >= board.board_width ||
                                 target_y < 0 || target_y >= board.board_height;
                }
                
                Token::EdgeTop(_, dy) => {
                    let target_y = board.piece_y + anchor_y + dy;
                    last_value = target_y >= board.board_height;
                }

                Token::EdgeBottom(_, dy) => {
                    let target_y = board.piece_y + anchor_y + dy;
                    last_value = target_y < 0;
                }
                
                Token::EdgeLeft(dx, _) => {
                    let target_x = board.piece_x + anchor_x + dx;
                    last_value = target_x < 0;
                }
                
                Token::EdgeRight(dx, _) => {
                    let target_x = board.piece_x + anchor_x + dx;
                    last_value = target_x >= board.board_width;
                }
                
                Token::Corner(dx, dy) => {
                    let target_x = board.piece_x + anchor_x + dx;
                    let target_y = board.piece_y + anchor_y + dy;
                    let out_x = target_x < 0 || target_x >= board.board_width;
                    let out_y = target_y < 0 || target_y >= board.board_height;
                    last_value = out_x && out_y;
                }
                
                Token::CornerTopLeft(dx, dy) => {
                    let target_x = board.piece_x + anchor_x + dx;
                    let target_y = board.piece_y + anchor_y + dy;
                    last_value = target_x < 0 && target_y >= board.board_height;
                }
                
                Token::CornerTopRight(dx, dy) => {
                    let target_x = board.piece_x + anchor_x + dx;
                    let target_y = board.piece_y + anchor_y + dy;
                    last_value = target_x >= board.board_width && target_y >= board.board_height;
                }
                
                Token::CornerBottomLeft(dx, dy) => {
                    let target_x = board.piece_x + anchor_x + dx;
                    let target_y = board.piece_y + anchor_y + dy;
                    last_value = target_x < 0 && target_y < 0;
                }
                
                Token::CornerBottomRight(dx, dy) => {
                    let target_x = board.piece_x + anchor_x + dx;
                    let target_y = board.piece_y + anchor_y + dy;
                    last_value = target_x >= board.board_width && target_y < 0;
                }
                
                // === 상태 ===
                Token::Piece(name) => {
                    last_value = board.piece_name == *name;
                }
                
                Token::IfState(key, expected) => {
                    let actual = *board.state.get(key).unwrap_or(&0);
                    last_value = actual == *expected;
                }
                
                Token::SetState(key, value) => {
                    pending_tags.push(ActionTag {
                        tag_type: ActionTagType::SetState,
                        key: key.clone(),
                        value: *value,
                        piece_name: None,
                    });
                    last_value = true;
                }
                
                Token::SetStateReset => {
                    pending_tags.pop();
                    last_value = true;
                }
                
                Token::Transition(piece_name) => {
                    pending_tags.push(ActionTag {
                        tag_type: ActionTagType::Transition,
                        key: String::new(),
                        value: 0,
                        piece_name: Some(piece_name.clone()),
                    });
                    last_value = true;
                }
                
                // === 제어 ===
                Token::Repeat(n) => {
                    // 앞의 n개 식으로 돌아가서 반복
                    if last_value && *n > 0 {
                        // 반복할 시작점 계산 (n개 토큰 전)
                        let target = if pc > *n { pc - *n - 1 } else { 0 };
                        pc = target;
                    }
                    // repeat은 last_value를 그대로 전달
                }
                
                Token::Do => {
                    // do는 일반 식 - false면 체인 종료
                    if last_value {
                        do_index = Some(pc);
                    }
                    // last_value 유지
                }
                
                Token::While => {
                    // while은 예외 - false여도 체인 종료 안함
                    if last_value {
                        if let Some(target) = do_index {
                            pc = target;
                        }
                    }
                    last_value = true;
                }
                
                Token::Jmp(label) => {
                    // 예외: false여도 종료 안함
                    if last_value {
                        let val_opt: usize = labels.get(&index_of_expression_chain).and_then(|inner| inner.get(label)).copied().expect("REASON");
                        pc = val_opt;
                    }
                    last_value = true;
                }
                
                Token::Jne(label) => {
                    // 예외: false면 점프, 체인 종료 안함
                    if !last_value {
                        let val_opt: usize = labels.get(&index_of_expression_chain).and_then(|inner| inner.get(label)).copied().expect("REASON");
                        pc = val_opt;
                    }
                    last_value = true;
                }
                
                Token::Label(_) => {
                    //skip
                }
                
                Token::Not => {
                    // 예외: 값 반전, 체인 종료 안함
                    last_value = !last_value;
                }
                
                Token::End => {
                    last_value = false;
                }
            }
        }
        
        activations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn make_empty_board() -> BoardState {
        BoardState {
            board_width: 8,
            board_height: 8,
            piece_x: 4,
            piece_y: 4,
            piece_name: "test".to_string(),
            is_white: true,
            pieces: HashMap::new(),
            state: HashMap::new(),
            danger_squares: std::collections::HashSet::new(),
            in_check: false,
        }
    }
    
    #[test]
    fn test_wazir() {
        // 와지르: 상하좌우 1칸
        let mut interp = Interpreter::new();
        interp.parse("take-move(1, 0); take-move(0, 1); take-move(-1, 0); take-move(0, -1);");
        let mut board = make_empty_board();
        let activations = interp.execute(&mut board);
        
        assert_eq!(activations.len(), 4);
        assert!(activations.iter().any(|a| a.dx == 1 && a.dy == 0));
        assert!(activations.iter().any(|a| a.dx == 0 && a.dy == 1));
        assert!(activations.iter().any(|a| a.dx == -1 && a.dy == 0));
        assert!(activations.iter().any(|a| a.dx == 0 && a.dy == -1));
    }
    
    #[test]
    fn test_rook_slide() {
        // 룩: 한 방향으로 슬라이드
        let mut interp = Interpreter::new();
        interp.parse("take-move(1, 0) repeat(1);");
        let mut board = make_empty_board();
        let activations = interp.execute(&mut board);
        
        // e5(4,4)에서 오른쪽으로 h5까지 3칸
        assert_eq!(activations.len(), 3);
        assert!(activations.iter().any(|a| a.dx == 1 && a.dy == 0));
        assert!(activations.iter().any(|a| a.dx == 2 && a.dy == 0));
        assert!(activations.iter().any(|a| a.dx == 3 && a.dy == 0));
    }
    
    #[test]
    fn test_rook_blocked_by_friendly() {
        let mut interp = Interpreter::new();
        interp.parse("take-move(1, 0) repeat(1);");
        let mut board = make_empty_board();
        // (6, 4)에 아군 배치
        board.pieces.insert((6, 4), ("pawn".to_string(), true));
        let activations = interp.execute(&mut board);
        
        // (5, 4)까지만 이동 가능 (dx=1)
        assert_eq!(activations.len(), 1);
        assert_eq!(activations[0].dx, 1);
    }
    
    #[test]
    fn test_rook_capture_enemy() {
        let mut interp = Interpreter::new();
        interp.parse("take-move(1, 0) repeat(1);");
        let mut board = make_empty_board();
        // (6, 4)에 적 배치
        board.pieces.insert((6, 4), ("pawn".to_string(), false));
        let activations = interp.execute(&mut board);
        
        // (5, 4)와 (6, 4) 모두 활성화
        assert_eq!(activations.len(), 2);
    }
    
    #[test]
    fn test_move_only() {
        // move는 빈 칸만
        let mut interp = Interpreter::new();
        interp.parse("move(1, 0);");
        let mut board = make_empty_board();
        board.pieces.insert((5, 4), ("enemy".to_string(), false));
        let activations = interp.execute(&mut board);
        
        // 적이 있으면 활성화 안됨
        assert_eq!(activations.len(), 0);
    }
    
    #[test]
    fn test_take_only() {
        // take는 적 있을 때만
        let mut interp = Interpreter::new();
        interp.parse("take(1, 0);");
        let mut board = make_empty_board();
        let activations = interp.execute(&mut board);
        
        // 빈 칸이면 활성화 안됨 (앵커만 이동)
        assert_eq!(activations.len(), 0);
    }
    
    #[test]
    fn test_scope_anchor_restore() {
        // { } 블록은 앵커를 복원
        let mut interp = Interpreter::new();
        interp.parse("move(0, 1) { move(1, 1) } move(-1, 1);");
        let mut board = make_empty_board();
        let activations = interp.execute(&mut board);
        
        // Y자 형태: (0,1), (1,2), (-1,2)
        assert_eq!(activations.len(), 3);
        assert!(activations.iter().any(|a| a.dx == 0 && a.dy == 1));
        assert!(activations.iter().any(|a| a.dx == 1 && a.dy == 2));
        assert!(activations.iter().any(|a| a.dx == -1 && a.dy == 2));
    }
    
    #[test]
    fn test_observe_blocked_knight() {
        // 장기 마: 막히면 못 감
        let mut interp = Interpreter::new();
        interp.parse("observe(1, 0) take-move(2, 1);");
        let mut board = make_empty_board();
        // (5, 4)에 기물 배치 - 막힘
        board.pieces.insert((5, 4), ("blocker".to_string(), true));
        let activations = interp.execute(&mut board);
        
        // observe가 false를 반환하여 take-move 실행 안됨
        assert_eq!(activations.len(), 0);
    }
    
    #[test]
    fn test_do_while_pattern() {
        // do...while 패턴
        let mut interp = Interpreter::new();
        interp.parse("do move(1, 0) while;");
        let mut board = make_empty_board();
        let activations = interp.execute(&mut board);
        
        // 오른쪽 끝까지 슬라이드
        assert_eq!(activations.len(), 3);
    }
    
    #[test]
    fn test_if_state() {
        let mut interp = Interpreter::new();
        interp.parse("if-state(mode, 0) move(1, 0);");
        let mut board = make_empty_board();
        // mode 기본값은 0
        let activations = interp.execute(&mut board);
        
        assert_eq!(activations.len(), 1);
    }
    
    #[test]
    fn test_if_state_false() {
        let mut interp = Interpreter::new();
        interp.parse("if-state(mode, 1) move(1, 0);");
        let mut board = make_empty_board();
        // mode는 0이므로 조건 불만족
        let activations = interp.execute(&mut board);
        
        assert_eq!(activations.len(), 0);
    }
    
    #[test]
    fn test_piece_condition() {
        let mut interp = Interpreter::new();
        interp.parse("piece(rook) move(1, 0);");
        let mut board = make_empty_board();
        board.piece_name = "rook".to_string();
        let activations = interp.execute(&mut board);
        
        assert_eq!(activations.len(), 1);
    }
    
    #[test]
    fn test_transition_tag() {
        let mut interp = Interpreter::new();
        interp.parse("transition(queen) move(1, 0);");
        let mut board = make_empty_board();
        let activations = interp.execute(&mut board);
        
        assert_eq!(activations.len(), 1);
        assert_eq!(activations[0].tags.len(), 1);
        assert_eq!(activations[0].tags[0].tag_type, ActionTagType::Transition);
        assert_eq!(activations[0].tags[0].piece_name, Some("queen".to_string()));
    }
    
    #[test]
    fn test_not() {
        let mut interp = Interpreter::new();
        // observe 결과를 not으로 반전
        interp.parse("observe(1, 0) not jne(SKIP) move(2, 0); label(SKIP) move(1, 0);");
        let mut board = make_empty_board();
        // (5,4)에 기물 있으면 observe=false, not=true, jne 안함, move(2,0) 실행
        board.pieces.insert((5, 4), ("blocker".to_string(), true));
        let activations = interp.execute(&mut board);
        
        // observe=false -> not=true -> jne 안함 -> move(2,0) 시도하지만 실패
        // 그래서 label(SKIP) move(1,0)도 별도 체인으로 실행됨
        assert!(activations.len() >= 1);
    }

    #[test]
    fn test_skip_chain_over_braces_until_semicolon() {
        let mut interp = Interpreter::new();
        interp.parse("if-state(mode, 1) set-state(mode, 0) { take-move(1, 0) repeat(1) } { take-move(-1, 0) repeat(1) };");
        let mut board = make_empty_board();
        // mode 기본 0이므로 조건 불만족 -> 모든 take-move는 무시되어야 함
        let activations = interp.execute(&mut board);
        assert_eq!(activations.len(), 0);
    }

    #[test]
    fn test_jmp(){
        let mut interp = Interpreter::new();
        interp.parse("piece(test) jmp(0) move(0, 1) label(0) piece(test) jmp(1) move(1, 0) move(1, 0) label(1); ");
        let mut board = make_empty_board();
        
        //piece(test)는 true이니 label로 점프 해야 함.
        let activations = interp.execute(&mut board);
        assert_eq!(activations.len(), 0);
    }

    #[test]
    fn test_jne(){
        let mut interp = Interpreter::new();
        interp.parse("piece(queen) jne(0) move(0, 1) label(0) move(1, 0) move(1, 0);");
        let mut board = make_empty_board();
        
        //piece(queen)는 false이니 label로 점프 해야 함.
        let activations = interp.execute(&mut board);
        assert_eq!(activations.len(), 2);
    }
}

 
