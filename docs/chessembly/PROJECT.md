## Chessembly의 모든 식(Expression)과 핵심 규칙의 요약

### 1. 식의 값 규칙

1. **`true` (계속):** 식이 성공하면 `true`를 반환하고, '식 연쇄'는 다음 식을 실행합니다.
2. **`false` (종료):** '일반 식'이 `false`를 반환하면, **'식 연쇄' 전체가 종료됩니다.**
3. **예외 5종:** `while`, `jmp`, `jne`, `not`, `label`은 `false`를 받아도 **연쇄를 종료시키지 않습니다.**

---

### 2. 행마식 (Movement Expressions)

칸을 활성화(🔵)하고 🌟 '기준 위치'를 이동시킵니다.

| **식 (Expression)** | **대상: 빈 칸 (Empty)** | **대상: 적 기물 (Enemy)** | **대상: 아군/벽 (Ally/Wall)** |
| --- | --- | --- | --- |
| **`move`** | 🔵 활성화, 기준 위치 이동, `true` | `false` (종료) | `false` (종료) |
| **`take`** | 기준 위치 이동, `true` | 🔵 활성화, 기준 위치 이동,  `true` | `false` (종료) |
| **`take-move`** | 🔵 활성화, 기준 위치 이동, `true` | 🔵 활성화, 기준 위치 이동, ❌ `false` (종료) | `false` (종료) |
| **`catch`** | 기준 위치 이동, `true` | 🔵 활성화, 기준 위치 이동, `true` | `false` (종료) |
| **`jump`** | 🔵 활성화, 기준 위치 이동, `true`  | `false` (종료) | `false` (종료) |
| **`shift`** | 🔵 활성화, 기준 위치 이동, `true` | 🔵 활성화, 기준 위치 이동, `true` | 아군인 경우 🔵 활성화, 기준 위치 이동 및 `true`, 벽인 경우 `false` |

---

### 3. 제어식 (Control Expressions)

'식 연쇄'의 실행 흐름(어떤 식이 다음에 실행될지)을 직접 제어합니다.

| 식 (Expression) | 직전 값이 false일 때 | 반환 값 | 설명 |
| --- | --- | --- | --- |
| **`repeat(n)`** | 연쇄 종료 | (직전 값) | `true`일 때만 `n`칸 뒤로 점프합니다. |
| **`{ ... }`** | 블록 종료 | (블록 마지막 값) | `false`를 격리하고 기준 위치를 복원합니다. (Y자 행마, 템페스트-룩) |
| **`end`** | (해당 없음) | (없음) | `{}` 블록 안에서도 '식 연쇄'를 무조건 종료합니다. |
| **`do`** | 연쇄 종료 | `true` | `while`과 쌍을 이루는 루프의 시작점. '일반 식'입니다. |
| **`while`** | 연쇄 계속 | **`true`** | **(예외 5종)** `true`일 때만 `do`로 점프합니다. (바운싱 비숍) |
| **`label(n)`** | 연쇄 계속 | (직전 값) | **(예외 5종)** `jmp`/`jne`의 목적지. 직전 값을 그대로 전달합니다. |
| **`jmp(n)`** | 연쇄 계속 | **`true`** | **(예외 5종)** `true`일 때만 `label(n)`으로 점프합니다. |
| **`jne(n)`** | 연쇄 계속 | **`true`** | **(예외 5종)** `false`일 때만 `label(n)`으로 점프합니다. (바운싱 비숍) |
| **`not`** | 연쇄 계속 | `! (직전 값)` | **(예외 5종)** `true`를 `false`로, `false`를 `true`로 뒤집습니다. |

---

### 4. 조건식 (Conditional Expressions)

칸을 활성화하지 않고, '엿보기'를 통해 ✅ `true` / ❌ `false`만 반환합니다. (모두 '일반 식'이므로 `false` 반환 시 연쇄가 종료됩니다.)

- `peek(dx, dy)`: (dx, dy)가 비어있으면 `true`를 반환하고, 기준 위치도 (dx, dy)만큼 이동합니다.
- `anchor(dx, dy)`: (dx, dy)가 체스판 안이라면 `true`를 반환하고, 기준 위치도 (dx, dy)만큼 이동합니다.
- `observe(dx, dy)`: (dx, dy)가 비어있으면 `true`를 반환합니다. (기준 위치 이동 안 함)
- `enemy(dx, dy)`: (dx, dy)에 적이 있으면 `true`를 반환합니다.
- `friendly(dx, dy)`: (dx, dy)에 아군이 있으면 `true`를 반환합니다.
- `piece-on(piece, dx, dy)`: (dx, dy)에 특정 `piece`가 있으면 `true`를 반환합니다.
- `danger(dx, dy)`: (dx, dy)가 적에게 공격받고 있으면 `true`를 반환합니다.
- `check`: 현재 아군이 체크 상태이면 `true`를 반환합니다.

### 경계 조건식 (Bounds)

- `bound(dx, dy)`: (dx, dy)가 보드 밖이면 `true`.
- `edge(dx, dy)`: (dx, dy)가 보드 변을 벗어나면 `true`.
- `corner(dx, dy)`: (dx, dy)가 보드 모서리를 벗어나면 `true`.
- `edge-(top|bottom|left|right)(dx, dy)`: 특정 방향의 변을 벗어나면 `true`. (바운싱 비숍)
- `corner-(top|bottom)-(left|right)(dx, dy)`: 특정 방향의 모서리를 벗어나면 `true`.

---

### 5. 상태식 (State Expressions)

게임의 '상태'를 읽거나, 이후 활성화될 칸에 특별한 효과를을 부여합니다.

- `piece(piece_name)`: (조건식) 이 코드를 실행하는 기물이 `piece_name`이면 `true`를 반환합니다. (Windmill 예제)
- `if-state(key, n)`: (조건식) 전역 변수 `key`의 값이 `n`이면 `true`를 반환합니다. (Windmill 예제)
- `transition(piece_name)`: (수식어) 이후 활성화되는 🔵칸에 "클릭 시 `piece_name`으로 변신" 액션을 부착합니다.
- `set-state(key, n)`: (수식어) 이후 활성화되는 🔵칸에 "클릭 시 `key` 값을 `n`으로 변경" 액션을 부착합니다.
- `set-state`: (수식어) `transition`이나 `set-state` 액션 부착을 비활성화합니다.

참고 (체인 독립성 및 수식어 범위):

- 한 연쇄(세미콜론으로 구분되는 하나의 식 연쇄)는 독립적으로 실행됩니다. 한 연쇄에서 활성화된 칸(🔵)이나 부착된 액션 태그는 다음 연쇄의 실행에 영향을 주지 않습니다.
- `set-state`나 `transition` 같은 수식어(액션 태그 부착)는 동일 연쇄 내에서만 유효합니다. 세미콜론으로 연쇄가 종료되면 해당 태그들은 다음 연쇄에 영향하지 않습니다.
- `set-state`(인자 없음)은 LIFO 방식으로 동작하여, 가장 마지막에 설정된 단일 액션 태그만 제거합니다.
