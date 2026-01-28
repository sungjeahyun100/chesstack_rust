import init, { Game } from '../pkg/chesstack_wasm.js';

let game;
let selectedSquare = null;
let legalMoves = [];
let selectedPocket = null; // { kind, owner }

// 기물 유니코드 매핑
const pieceSymbols = {
    king: { white: '♔', black: '♚' },
    queen: { white: '♕', black: '♛' },
    rook: { white: '♖', black: '♜' },
    bishop: { white: '♗', black: '♝' },
    knight: { white: '♘', black: '♞' },
    pawn: { white: '♙', black: '♟' },
    amazon: { white: 'A', black: 'a' },
    grasshopper: { white: 'G', black: 'g' },
    archbishop: { white: 'Ab', black: 'ab' },
    knightrider: { white: 'Kr', black: 'kr' },
    tempestrook: { white: 'Tr', black: 'tr' },
    dabbaba: { white: 'Da', black: 'da' },
    alfil: { white: 'Al', black: 'al' },
    ferz: { white: 'Fz', black: 'fz' },
    centaur: { white: 'Ce', black: 'ce' },
    camel: { white: 'Ca', black: 'ca' },
    cannon: {white: 'Cn', black: "cn"},
    // Custom 기물은 fallback '?' 처리
};

async function main() {
    await init();
    game = new Game();
    game.setup_initial();
    render();
}

function render() {
    renderBoard();
    renderPockets();
    updateTurnIndicator();
}

function renderBoard() {
    const board = document.getElementById('board');
    board.innerHTML = '';

    const state = game.get_state();

    // y=7이 위 (흑 진영), y=0이 아래 (백 진영)
    for (let y = 7; y >= 0; y--) {
        for (let x = 0; x < 8; x++) {
            const square = document.createElement('div');
            const isLight = (x + y) % 2 === 1;
            square.className = `square ${isLight ? 'light' : 'dark'}`;
            square.dataset.x = x;
            square.dataset.y = y;

            // 선택된 칸 표시
            if (selectedSquare && selectedSquare.x === x && selectedSquare.y === y) {
                square.classList.add('selected');
            }

            // 이동 가능한 칸 표시
            const legalMove = legalMoves.find(m => m.to_x === x && m.to_y === y);
            if (legalMove) {
                // MoveType에 따라 다른 스타일 적용
                square.classList.add('legal-move');
                square.classList.add(`move-type-${legalMove.move_type.toLowerCase()}`);
                
                // 시각적 표시 추가
                const indicator = document.createElement('div');
                indicator.className = 'move-indicator-dot';
                
                switch(legalMove.move_type) {
                    case 'Move':
                        indicator.textContent = '○'; // 빈 원 - 이동만
                        break;
                    case 'Take':
                        indicator.textContent = '×'; // X - 잡기만
                        break;
                    case 'TakeMove':
                        indicator.textContent = legalMove.is_capture ? '⊗' : '●'; // 채운 원 - 이동/잡기
                        break;
                    case 'Catch':
                        indicator.textContent = '⊕'; // 십자 원 - 원거리 잡기
                        break;
                    case 'Shift':
                        indicator.textContent = '⇄'; // 양방향 화살표 - 자리바꾸기
                        break;
                    case 'Jump':
                        indicator.textContent = '◇'; // 다이아몬드 - 점프
                        break;
                }
                
                square.appendChild(indicator);
            }

            // 기물 렌더링
            const piece = state.pieces.find(p => p.x === x && p.y === y);
            if (piece) {
                const pieceEl = document.createElement('div');
                const color = piece.owner === 0 ? 'white' : 'black';
                pieceEl.className = `piece ${color}`;

                const symbols = pieceSymbols[piece.kind];
                pieceEl.textContent = symbols ? symbols[color] : '?';

                square.appendChild(pieceEl);

                // 스턴 표시
                if (piece.stun_stack > 0) {
                    const stunEl = document.createElement('div');
                    stunEl.className = 'stun-indicator';
                    stunEl.textContent = piece.stun_stack;
                    square.appendChild(stunEl);
                }

                // 이동 스택 표시 
                if (piece.move_stack > 0) {
                    const moveEl = document.createElement('div');
                    moveEl.className = 'move-indicator';
                    moveEl.textContent = piece.move_stack;
                    square.appendChild(moveEl);
                }
            }

            square.addEventListener('click', () => onSquareClick(x, y));
            board.appendChild(square);
        }
    }
}

function onSquareClick(x, y) {
    const state = game.get_state();
    const piece = state.pieces.find(p => p.x === x && p.y === y);
    const isOccupied = Boolean(piece);

    // 포켓 선택 상태에서 빈 칸 클릭 시 착수 시도
    if (selectedPocket && !isOccupied) {
        const success = game.place_from_pocket(selectedPocket.kind, x, y);
        if (success) {
            console.log(`Placed ${selectedPocket.kind} at (${x}, ${y})`);
            selectedPocket = null;
            selectedSquare = null;
            legalMoves = [];
            render();
            return;
        }
    }

    // 이동 가능한 칸 클릭 시 이동
    if (selectedSquare && legalMoves.some(m => m.to_x === x && m.to_y === y)) {
        const success = game.move_piece(selectedSquare.x, selectedSquare.y, x, y);
        if (success) {
            console.log(`Moved from (${selectedSquare.x}, ${selectedSquare.y}) to (${x}, ${y})`);
        }
        selectedSquare = null;
        legalMoves = [];
        render();
        checkGameOver();
        return;
    }

    // 자신의 기물 클릭 시 선택
    if (piece && piece.owner === state.current_player) {
        selectedSquare = { x, y };
        legalMoves = game.get_legal_moves(x, y);
        console.log(`Selected piece at (${x}, ${y}), legal moves:`, legalMoves);
        // MoveType 요약 출력
        const moveTypes = legalMoves.reduce((acc, m) => {
            acc[m.move_type] = (acc[m.move_type] || 0) + 1;
            return acc;
        }, {});
        console.log('Move types:', moveTypes);
    } else {
        selectedSquare = null;
        legalMoves = [];
    }

    render();
}

function renderPockets() {
    const state = game.get_state();

    const whitePocket = document.getElementById('whitePocket');
    whitePocket.innerHTML = '';
    for (const kind of state.white_pocket) {
        const el = document.createElement('div');
        el.className = 'pocket-piece';
        const symbols = pieceSymbols[kind];
        el.textContent = symbols ? symbols.white : '?';
        if (selectedPocket && selectedPocket.kind === kind && selectedPocket.owner === 0) {
            el.classList.add('selected');
        }
        el.addEventListener('click', () => {
            if (state.current_player !== 0) return;
            selectedPocket = { kind, owner: 0 };
            selectedSquare = null;
            legalMoves = [];
            render();
        });
        whitePocket.appendChild(el);
    }

    const blackPocket = document.getElementById('blackPocket');
    blackPocket.innerHTML = '';
    for (const kind of state.black_pocket) {
        const el = document.createElement('div');
        el.className = 'pocket-piece';
        const symbols = pieceSymbols[kind];
        el.textContent = symbols ? symbols.black : '?';
        if (selectedPocket && selectedPocket.kind === kind && selectedPocket.owner === 1) {
            el.classList.add('selected');
        }
        el.addEventListener('click', () => {
            if (state.current_player !== 1) return;
            selectedPocket = { kind, owner: 1 };
            selectedSquare = null;
            legalMoves = [];
            render();
        });
        blackPocket.appendChild(el);
    }
}

function updateTurnIndicator() {
    const state = game.get_state();
    const indicator = document.getElementById('turnIndicator');
    indicator.innerHTML = state.current_player === 0 ? '⚪ 백 차례' : '⚫ 흑 차례';
}

function checkGameOver() {
    if (game.is_game_over()) {
        const winner = game.winner();
        alert(winner === 1 ? '백 승리!' : '흑 승리!');
    }
}

function endTurn() {
    game.end_turn();
    selectedSquare = null;
    legalMoves = [];
    selectedPocket = null;
    render();
}

function newGame() {
    game = new Game();
    game.setup_initial();
    selectedSquare = null;
    legalMoves = [];
    selectedPocket = null;
    render();
}

function experimentalGame() {
    game = new Game();
    game.setup_experimental();
    selectedSquare = null;
    legalMoves = [];
    selectedPocket = null;
    render();
}

// expose globals for buttons
window.endTurn = endTurn;
window.newGame = newGame;
window.experimentalGame = experimentalGame;

main();
