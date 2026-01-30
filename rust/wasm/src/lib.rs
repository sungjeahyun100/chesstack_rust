use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use engine::{GameState, Square, PieceKind, Action, PlayerId, GameResult};

/// JS에서 사용할 게임 래퍼
#[wasm_bindgen]
pub struct Game {
    state: GameState,
}

/// JS로 전달할 기물 정보
#[derive(Serialize, Deserialize)]
pub struct JsPiece {
    pub id: String,
    pub kind: String,
    pub owner: u8,
    pub x: i32,
    pub y: i32,
    pub stun_stack: i32,
    pub move_stack: i32,
    pub is_royal: bool,
}

/// JS로 전달할 이동 정보
#[derive(Serialize, Deserialize)]
pub struct JsMove {
    pub from_x: i32,
    pub from_y: i32,
    pub to_x: i32,
    pub to_y: i32,
    pub is_capture: bool,
    pub move_type: String, // "TakeMove", "Move", "Take", "Catch", "Shift", "Jump"
}

/// JS로 전달할 게임 상태
#[derive(Serialize, Deserialize)]
pub struct JsGameState {
    pub pieces: Vec<JsPiece>,
    pub current_player: u8,
    pub white_pocket: Vec<String>,
    pub black_pocket: Vec<String>,
    pub is_game_over: bool,
    pub winner: Option<u8>,
}

#[wasm_bindgen]
impl Game {
    /// 새 게임 생성
    #[wasm_bindgen(constructor)]
    pub fn new() -> Game {
        Game {
            state: GameState::new_default(),
        }
    }
    
    /// 초기 배치로 게임 시작
    #[wasm_bindgen]
    pub fn setup_initial(&mut self) {
        self.state.setup_initial_position();
    }
    
    /// 실험용 포켓으로 게임 시작
    #[wasm_bindgen]
    pub fn setup_experimental(&mut self) {
        self.state.setup_experimental_pocket();
    }
    
    /// 디버그 모드 설정 (Chessembly 실행 추적)
    #[wasm_bindgen]
    pub fn set_debug(&mut self, enabled: bool) {
        self.state.debug_mode = enabled;
    }
    
    /// 현재 게임 상태를 JSON으로 반환
    #[wasm_bindgen]
    pub fn get_state(&self) -> JsValue {
        let js_state = self.build_js_state();
        serde_wasm_bindgen::to_value(&js_state).unwrap()
    }
    
    /// 특정 칸의 기물이 갈 수 있는 칸 목록
    #[wasm_bindgen]
    pub fn get_legal_moves(&self, x: i32, y: i32) -> JsValue {
        let square = Square::new(x, y);
        let moves = self.state.get_legal_moves_at(square);
        
        let js_moves: Vec<JsMove> = moves.iter().map(|m| {
            let move_type_str = match m.move_type {
                engine::MoveType::TakeMove => "TakeMove",
                engine::MoveType::Move => "Move",
                engine::MoveType::Take => "Take",
                engine::MoveType::Catch => "Catch",
                engine::MoveType::Shift => "Shift",
                engine::MoveType::Jump => "Jump",
            };
            
            JsMove {
                from_x: m.from.x,
                from_y: m.from.y,
                to_x: m.to.x,
                to_y: m.to.y,
                is_capture: m.is_capture,
                move_type: move_type_str.to_string(),
            }
        }).collect();
        
        serde_wasm_bindgen::to_value(&js_moves).unwrap()
    }
    
    /// 기물 이동 실행
    #[wasm_bindgen]
    pub fn move_piece(&mut self, from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> bool {
        let from = Square::new(from_x, from_y);
        let to = Square::new(to_x, to_y);
        
        if self.state.is_valid_move_at(from, to) {
            // 이동 액션 실행
            if let Some(piece) = self.state.get_piece_at(from) {
                let action = Action::Move { 
                    piece_id: piece.id.clone(), 
                    from, 
                    to,
                };
                self.state.apply_action(action);
                return true;
            }
        }
        false
    }
    
    /// 포켓에서 기물 배치 (간단화된 버전 - 실제 구현 필요)
    #[wasm_bindgen]
    pub fn place_from_pocket(&mut self, kind: &str, x: i32, y: i32) -> bool {
        let piece_kind = self.parse_piece_kind(kind);
        let square = Square::new(x, y);

        if self.state.can_place_from_pocket(&piece_kind, square) {
            if self.state.place_piece(self.state.current_player(), piece_kind.clone(), square).is_ok() {
                return true;
            }
        }

        false
    }
    
    /// 턴 종료
    #[wasm_bindgen]
    pub fn end_turn(&mut self) {
        self.state.end_turn();
    }
    
    /// 현재 플레이어
    #[wasm_bindgen]
    pub fn current_player(&self) -> u8 {
        self.state.current_player()
    }
    
    /// 게임 종료 여부
    #[wasm_bindgen]
    pub fn is_game_over(&self) -> bool {
        !matches!(self.state.check_victory(), GameResult::Ongoing)
    }
    
    /// 승자 (0=진행중, 1=백, 2=흑)
    #[wasm_bindgen]
    pub fn winner(&self) -> u8 {
        match self.state.check_victory() {
            GameResult::WhiteWins => 1,
            GameResult::BlackWins => 2,
            GameResult::Ongoing => 0,
        }
    }
    
    // === Private helpers ===
    
    fn build_js_state(&self) -> JsGameState {
        let pieces: Vec<JsPiece> = self.state.get_all_pieces().iter().map(|p| {
            JsPiece {
                id: p.id.clone(),
                kind: self.kind_to_string(&p.kind),
                owner: p.owner,
                x: p.pos.x,
                y: p.pos.y,
                stun_stack: p.stun_stack,
                move_stack: p.move_stack,
                is_royal: p.is_royal,
            }
        }).collect();
        
        let victory = self.state.check_victory();
        JsGameState {
            pieces,
            current_player: self.state.current_player(),
            white_pocket: self.pocket_to_strings(0),
            black_pocket: self.pocket_to_strings(1),
            is_game_over: !matches!(victory, GameResult::Ongoing),
            winner: match victory {
                GameResult::WhiteWins => Some(1),
                GameResult::BlackWins => Some(2),
                GameResult::Ongoing => None,
            },
        }
    }
    
    fn kind_to_string(&self, kind: &PieceKind) -> String {
        match kind {
            PieceKind::Pawn => "pawn".to_string(),
            PieceKind::King => "king".to_string(),
            PieceKind::Queen => "queen".to_string(),
            PieceKind::Rook => "rook".to_string(),
            PieceKind::Knight => "knight".to_string(),
            PieceKind::Bishop => "bishop".to_string(),
            PieceKind::Amazon => "amazon".to_string(),
            PieceKind::Grasshopper => "grasshopper".to_string(),
            PieceKind::Knightrider => "knightrider".to_string(),
            PieceKind::Archbishop => "archbishop".to_string(),
            PieceKind::Dabbaba => "dabbaba".to_string(),
            PieceKind::Alfil => "alfil".to_string(),
            PieceKind::Ferz => "ferz".to_string(),
            PieceKind::Centaur => "centaur".to_string(),
            PieceKind::Camel => "camel".to_string(),
            PieceKind::TempestRook => "tempestrook".to_string(),
            PieceKind::Cannon => "cannon".to_string(),
            PieceKind::Experiment => "experiment".to_string(),
            PieceKind::Custom(s) => s.clone(),
        }
    }
    
    fn parse_piece_kind(&self, s: &str) -> PieceKind {
        match s.to_lowercase().as_str() {
            "pawn" => PieceKind::Pawn,
            "king" => PieceKind::King,
            "queen" => PieceKind::Queen,
            "rook" => PieceKind::Rook,
            "knight" => PieceKind::Knight,
            "bishop" => PieceKind::Bishop,
            "amazon" => PieceKind::Amazon,
            "grasshopper" => PieceKind::Grasshopper,
            "knightrider" => PieceKind::Knightrider,
            "archbishop" => PieceKind::Archbishop,
            "dabbaba" => PieceKind::Dabbaba,
            "alfil" => PieceKind::Alfil,
            "ferz" => PieceKind::Ferz,
            "centaur" => PieceKind::Centaur,
            "camel" => PieceKind::Camel,
            "tempestrook" => PieceKind::TempestRook,
            "cannon" => PieceKind::Cannon,
            "experiment" => PieceKind::Experiment,
            _ => PieceKind::Custom(s.to_string()),
        }
    }
    
    fn pocket_to_strings(&self, player: PlayerId) -> Vec<String> {
        self.state.get_pocket(player).iter()
            .map(|k| self.kind_to_string(k))
            .collect()
    }
}

/// 콘솔 로그 (디버깅용)
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
pub fn main() {
    log("Chesstack WASM initialized!");
}
