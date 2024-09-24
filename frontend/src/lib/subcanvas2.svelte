<script lang="ts">
  import axios from "axios";
  import { onMount, tick } from "svelte";

  let canvas: HTMLCanvasElement;
  let hoverSquare: HTMLDivElement;
  let ctx: CanvasRenderingContext2D;
  let socket: WebSocket;

  let innerWidth: number;
  let innerHeight: number;

  let xShift = 0;
  let yShift = 0;
  let fullWidth = 0;

  onMount(async () => {
    ctx = canvas.getContext("2d")!!;
    await tick();
    resizeCanvas();
    ws();
  });

  const numCols = 80;
  const numRows = 80;

  let squareSize: number; // We'll calculate this dynamically

  let squares = Array(numRows)
    .fill()
    .map(() => Array(numCols).fill("white"));

  // Function to resize canvas and recalculate the grid size
  function resizeCanvas() {
    // Set canvas size to window size
    canvas.width = innerWidth; //window.innerWidth;
    canvas.height = innerHeight; //window.innerHeight;

    // Calculate square size based on new canvas dimensions
    squareSize = Math.min(canvas.width / numCols, canvas.height / numRows);

    // Update hover square size
    hoverSquare.style.width = `${squareSize}px`;
    hoverSquare.style.height = `${squareSize}px`;

    // Redraw the grid
    renderGrid();
  }

  // Function to render the entire grid
  function renderGrid() {
    for (let row = 0; row < numRows; row++) {
      for (let col = 0; col < numCols; col++) {
        let color = squares[row][col];
        renderSquare(row, col, color);
      }
    }
  }

  // Function to render a single square
  function renderSquare(row: number, col: number, color: string) {
    const x = col * squareSize;
    const y = row * squareSize;

    ctx.fillStyle = color;
    ctx.strokeStyle = "gray";
    ctx.lineWidth = 0.5;
    ctx.fillRect(x, y, squareSize, squareSize);
    ctx.strokeRect(x, y, squareSize, squareSize);
  }

  function handleCanvasClick(event: MouseEvent) {
    const rect = canvas.getBoundingClientRect();
    const localX = event.clientX - rect.left;
    const localY = event.clientY - rect.top;

    // Determine the clicked square
    const col = Math.floor(localX / squareSize);
    const row = Math.floor(localY / squareSize);
    if (col >= 80 || col < 0) {
      return;
    }
    if (row >= 80 || row < 0) {
      return;
    }

    let x = col;
    let y = row;

    let globalX = xShift + x;
    let globalY = yShift + y;
    // console.log("global", globalX, globalY);
    let index = globalY * 1000 + globalX;
    // console.log("Index", index);

    axios.post("/set/" + index.toString()).then((d) => {
      let color;
      if (d.data == "1") {
        color = "#000000";
      } else {
        color = "#00ff00";
      }

      // Change the color of the clicked square
      squares[row][col] = color;
      // Re-render only the clicked square
      renderSquare(row, col, color);
    });
  }

  function handleCanvasMouseLeave() {
    hoverSquare.style.display = "none";
  }

  function handleCanvasMouseMove(event: MouseEvent) {
    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;

    // Determine the hovered square
    const col = Math.floor(x / squareSize);
    const row = Math.floor(y / squareSize);

    if (col >= 80 || col < 0) {
      handleCanvasMouseLeave();
      return;
    }
    if (row >= 80 || row < 0) {
      handleCanvasMouseLeave();
      return;
    }

    // Position the hover square (DOM element)
    hoverSquare.style.left = `${col * squareSize + rect.left}px`;
    hoverSquare.style.top = `${row * squareSize + rect.top}px`;
    hoverSquare.style.display = "block"; // Show the hover square
  }

  async function loadCanvas() {
    let response = await axios.get("/api/subgrid");
    // handle success
    //console.log(response);
    let subgridData = response.data;

    let data = base64ToArrayBuffer(subgridData.data);
    //console.log(data);
    xShift = subgridData.x_shift;
    yShift = subgridData.y_shift;
    fullWidth = subgridData.canvas_width;
    await tick();
    loadInitialCanvasData(data, subgridData.width, subgridData.height);
  }

  function loadInitialCanvasData(
    inputData: ArrayBufferLike,
    width: number,
    height: number
  ) {
    let setCount = 0;
    //const imgData = ctx.createImageData(width, height);
    let byteInputData = new Uint8Array(inputData);
    // console.log(byteInputData.length);
    //const data = imgData.data;
    let bit_index = 0;
    let byte_index = 0;
    for (let i = 0; i < byteInputData.length * 8; i++) {
      let x = i % width;
      let y = Math.floor(i / width);
      bit_index = i % 8;
      byte_index = Math.floor(i / 8);
      let byte_value = byteInputData[byte_index];
      let is_set = is_bit_set(byte_value, bit_index);
      if (is_set) {
        setCount += 1;
        squares[y][x] = "red";
        renderSquare(y, x, "red");
      } else {
        squares[y][x] = "white";
        renderSquare(y, x, "white");
      }
    }

    console.log(`${setCount} pixels set`);
  }

  function base64ToArrayBuffer(base64: string) {
    var binaryString = atob(base64);
    var bytes = new Uint8Array(binaryString.length);
    for (var i = 0; i < binaryString.length; i++) {
      bytes[i] = binaryString.charCodeAt(i);
    }
    return bytes.buffer;
  }

  function is_bit_set(byte_value: number, bit_index: number) {
    let mask = 1 << bit_index;
    return (byte_value & mask) != 0;
  }

  function ws() {
    const protocol = window.location.protocol === "https:" ? "wss://" : "ws://";
    const hostname = window.location.hostname;
    const port = window.location.port ? `:${window.location.port}` : ""; // If there's a port, include it
    const path = "/ws"; // Replace with your actual path

    const wsUrl = `${protocol}${hostname}${port}${path}`;
    socket = new WebSocket(wsUrl);
    console.log("Connecting");

    socket.onmessage = (event) => {
      //console.log(`[message] Данные получены с сервера: ${event.data}`);
      let data = event.data;
      let color;
      data = JSON.parse(data);
      data.on.forEach((index: number) => {
        color = "#ff0000";
        let y = Math.floor(index / 1000) - yShift;
        let x = (index % 1000) - xShift;
        // console.log(x, y);
        if (x < 0 || x >= 80) {
          return;
        }
        if (x < 0 || y >= 80) {
          return;
        }
        squares[y][x] = color;
        renderSquare(y, x, color);
      });
      data.off.forEach((index: number) => {
        color = "#ffffff";
        let y = Math.floor(index / 1000) - yShift;
        let x = (index % 1000) - xShift;
        if (x < 0 || x >= 80) {
          return;
        }
        if (x < 0 || y >= 80) {
          return;
        }
        console.log(x, y);
        squares[y][x] = color;
        renderSquare(y, x, color);
      });
    };

    socket.onopen = async (e) => {
      console.log("[open] Соединение установлено");
      await loadCanvas();
    };

    socket.onclose = (event) => {
      if (event.wasClean) {
        console.log(
          `[close] Соединение закрыто чисто, код=${event.code} причина=${event.reason}`
        );
      } else {
        console.log("[close] Соединение прервано");
      }
    };

    socket.onerror = (error) => {
      console.log(`[error]`, error);
    };
  }
</script>

<svelte:window bind:innerHeight bind:innerWidth on:resize={resizeCanvas} />

<canvas
  id="canvas"
  bind:this={canvas}
  on:click={handleCanvasClick}
  on:mouseleave={handleCanvasMouseLeave}
  on:mousemove={handleCanvasMouseMove}
></canvas>
<div id="hover-square" bind:this={hoverSquare}></div>

<style>
  #hover-square {
    position: absolute;
    border: 1px solid black;
    background-color: orange;
    pointer-events: none; /* Ignore mouse events for this element */
    display: none; /* Hide initially */
  }
</style>
