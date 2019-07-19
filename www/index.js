import { Universe } from "wasm-game-of-life";
import { memory } from "wasm-game-of-life/wasm_game_of_life_bg";

const CELL_SIZE = 5;
const GRID_COLOR = "#CCCCCC";
const DEAD_COLOR = "#FFFFFF";
const ALIVE_COLOR = "#000000";

const [width, height] = [128, 128];
const universe = Universe.new(width, height);
const canvas = document.getElementById("game-of-life-canvas");
canvas.height = ((CELL_SIZE + 1) * height + 1) / 1.5;
canvas.width = ((CELL_SIZE + 1) * width + 1) / 1.5;

const ctx = canvas.getContext('2d');

const drawGrid = () => {
    ctx.beginPath();
    ctx.strokeStyle = GRID_COLOR;

    // Vertical lines.
    for (let i = 0; i <= width; i++) {
        ctx.moveTo(i * (CELL_SIZE + 1) + 1, 0);
        ctx.lineTo(i * (CELL_SIZE + 1) + 1, canvas.height);
    }

    // Horizontal lines.
    for (let j = 0; j <= height; j++) {
        ctx.moveTo(0, j * (CELL_SIZE + 1) + 1);
        ctx.lineTo(canvas.width, j * (CELL_SIZE + 1) + 1);
    }

    ctx.stroke();
};

const getIndex = (row, col) => row * width + col;
const bitIsSet = (n, arr) => {
    const mask = 1 << (n % 8);
    const byte = Math.floor(n / 8);
    return (arr[byte] & mask) === mask;
}

const drawCells = () => {
    // A pointer to the cells in wasm's linear memory.
    const cellsPtr = universe.cells();

    // Each cell takes 1 bit in memory.
    const cells = new Uint8Array(memory.buffer, cellsPtr, width * height / 8);

    // Draw cells as squares in the grid.
    ctx.beginPath();
    for (let row = 0; row < height; row++) {
        for (let col = 0; col < width; col++) {
            const idx = getIndex(row, col);

            ctx.fillStyle = bitIsSet(idx, cells)
                ? ALIVE_COLOR
                : DEAD_COLOR;

            ctx.fillRect(
                col * (CELL_SIZE + 1) + 1,
                row * (CELL_SIZE + 1) + 1,
                CELL_SIZE,
                CELL_SIZE,
            );
        }
    }

    ctx.stroke();
}

canvas.addEventListener('click', event => {
    const boundingRect = canvas.getBoundingClientRect();
    const scaleX = canvas.width / boundingRect.width;
    const scaleY = canvas.height / boundingRect.height;

    const canvasLeft = (event.clientX - boundingRect.left) * scaleX;
    const canvasTop = (event.clientY - boundingRect.top) * scaleY;

    const row = Math.min(Math.floor(canvasTop / (CELL_SIZE + 1)), height - 1);
    const col = Math.min(Math.floor(canvasLeft / (CELL_SIZE + 1)), width - 1);

    if (event.shiftKey) {
        universe.put_glider(row, col);
    } else if (event.altKey) {
        universe.put_pulsar(row, col);
    } else {
        universe.toggle_cell(row, col);
    }

    drawGrid();
    drawCells();
});

let animationId = null;
const ticksRange = document.getElementById('ticks-per-frame');
ticksRange.addEventListener('change', () => console.log(`ticks per frame: ${ticksRange.value}`));
const renderLoop = () => {
    for (let i = 0; i < ticksRange.value; i++) {
        universe.tick();
    }
    drawGrid();
    drawCells();
    animationId = requestAnimationFrame(renderLoop);
};

const playPauseButton = document.getElementById('play-pause');
const play = () => {
    playPauseButton.textContent = '⏸';
    renderLoop();
}

const pause = () => {
    playPauseButton.textContent = "▶";
    cancelAnimationFrame(animationId);
    animationId = null;
}

const isPaused = () => animationId === null;

playPauseButton.addEventListener('click', event => {
    if (isPaused()) {
        play();
    } else {
        pause();
    }
});

const resetButton = document.getElementById('reset');
resetButton.addEventListener('click', () => {
    universe.reset();
    drawGrid();
    drawCells();
});

drawGrid();
drawCells();
play();
