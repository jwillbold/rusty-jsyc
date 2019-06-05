var canvas = document.getElementById("canvas");
var ctx = canvas.getContext("2d");

const bh = canvas.height;
const bw = canvas.width;

const hFieldCount = 50;
const vFieldCount = 40;
const fieldLength = bh/vFieldCount

function rand(min, max) {
  return Math.floor((Math.random() * max) + min);
}

function max(a, b) {
  return a >= b ? a : b;
}

function fillField(x, y, color) {
  ctx.beginPath();
  ctx.rect(x*fieldLength+1, y*fieldLength+1, fieldLength-1, fieldLength-1);
  ctx.fillStyle = color;
  ctx.fill();
  ctx.closePath();
}

var apple_x;
var apple_y;

function spawnNewApple() {
  apple_x = rand(0, hFieldCount-1);
  apple_y = rand(0, vFieldCount-1);
}

spawnNewApple();

const DIRECTION_UP = 0;
const DIRECTION_DOWN = 1;
const DIRECTION_LEFT = 2;
const DIRECTION_RIGHT = 3;

var snake_fields =[[hFieldCount/2, vFieldCount/2, DIRECTION_LEFT],
                  [hFieldCount/2+1, vFieldCount/2, DIRECTION_LEFT],
                  [hFieldCount/2+2, vFieldCount/2, DIRECTION_LEFT]];

var score = 0;
var snake_direction = DIRECTION_LEFT;
var updateSpeed = 100;
var updater;

function drawGrid() {
  ctx.beginPath();
  for (var x = fieldLength; x <= bw; x += fieldLength) {
      ctx.moveTo(0.5 + x, 0);
      ctx.lineTo(0.5 + x, bh);
  }
  for (var y = fieldLength; y <= bh; y += fieldLength) {
      ctx.moveTo(0, 0.5 + y);
      ctx.lineTo(bw, 0.5 + y);
  }
  ctx.strokeStyle = "grey";
  ctx.stroke();
  ctx.closePath();
}

function drawFrame() {
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  drawGrid();

  for(var i = 0; i<snake_fields.length; ++i) {
    var field = snake_fields[i];
    fillField(field[0], field[1], "orange");
  }

  fillField(apple_x, apple_y, "green");

  if(updater) {
    requestAnimationFrame(drawFrame);
  }
}

function updateTick() {
  const dx = [0, 0, -1, 1];
  const dy = [-1, 1, 0, 0];
  var lastDirection = snake_direction;
  var newField = void 0;

  for(var i = 0; i<snake_fields.length; ++i) {
    var field = snake_fields[i];

    if(field[2] != lastDirection) {
      field[2] = [lastDirection, lastDirection = field[2]][0];
    }

    var x = field[0] + dx[field[2]];
    var y = field[1] + dy[field[2]];

    if(x == apple_x && y == apple_y) {
      newField = Object.assign([], snake_fields[snake_fields.length-1]);
      spawnNewApple();
      ++score;
    }

    let collisionCounter = 0;
    for(var ci = 0; ci<snake_fields.length; ++ci) {
      var collisionTestField = snake_fields[ci];
      if((collisionTestField[0] === field[0]) && (collisionTestField[1] === field[1])) {
        ++collisionCounter;
      }
    }

    if(x < 0 || x >= hFieldCount || y < 0 || y >= vFieldCount || collisionCounter >= 2) {
      // game over
      document.location.reload();
    }

    snake_fields[i] = [x, y, field[2]];
  }

  if(newField) {
    snake_fields.push(newField);
  }
}

function startOrContinue() {
  updater = setInterval(updateTick, updateSpeed);
  drawFrame();
}

function toggleUpdateLoop() {
  if(updater) {
    clearInterval(updater);
    updater = void 0;
  } else {
    startOrContinue();
  }
}

function eventHandler(event) {
  var maybe_current_dir, new_dir;

  if(event.key == "ArrowLeft") {
    new_dir = DIRECTION_LEFT;
    maybe_current_dir = DIRECTION_RIGHT;
  } else if(event.key == "ArrowRight") {
    new_dir = DIRECTION_RIGHT;
    maybe_current_dir = DIRECTION_LEFT;
  } else if(event.key == "ArrowUp") {
    new_dir = DIRECTION_UP;
    maybe_current_dir = DIRECTION_DOWN;
  } else if(event.key == "ArrowDown") {
    new_dir = DIRECTION_DOWN;
    maybe_current_dir = DIRECTION_UP;
  }

  if((maybe_current_dir !== void 0) && (maybe_current_dir != snake_direction)) {
    snake_direction = new_dir;
  } else if(event.key == " ") {
    toggleUpdateLoop();
  } else if(event.key == "+") {
    updateSpeed = max(updateSpeed-10, 20);
    toggleUpdateLoop();
    toggleUpdateLoop();
  }
}

document.addEventListener("keydown", eventHandler, false);

startOrContinue();
