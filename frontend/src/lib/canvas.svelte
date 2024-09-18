<script lang="ts">
  import axios from "axios";
  import panzoom, { type PanZoom } from "panzoom";
  import { onDestroy, onMount } from "svelte";

  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D;
  let socket: WebSocket;
  let instance: PanZoom;

  onMount(() => {
    ctx = canvas.getContext("2d")!!;
    ws();
  });

  onDestroy(() => {});

  function setPixel(x: number, y: number, color: string) {
    // Create an ImageData object with a single pixel
    const imgData = ctx.createImageData(1, 1);
    const data = imgData.data;

    // Parse the color string and set the pixel data
    // Assuming color is in the format '#RRGGBB'
    const r = parseInt(color.slice(1, 3), 16);
    const g = parseInt(color.slice(3, 5), 16);
    const b = parseInt(color.slice(5, 7), 16);
    const a = 255; // Fully opaque

    // Set the pixel color (R, G, B, A)
    data[0] = r;
    data[1] = g;
    data[2] = b;
    data[3] = a;

    // Put the ImageData object onto the canvas at (x, y)
    ctx.putImageData(imgData, x, y);
  }

  function loadCanvas() {
    axios.get("/grid").then(function (response) {
      // handle success
      //console.log(response);
      let data = base64ToArrayBuffer(response.data);
      //console.log(data);
      loadInitialCanvasData(data);
    });
  }

  function base64ToArrayBuffer(base64: string) {
    var binaryString = atob(base64);
    var bytes = new Uint8Array(binaryString.length);
    for (var i = 0; i < binaryString.length; i++) {
      bytes[i] = binaryString.charCodeAt(i);
    }
    return bytes.buffer;
  }

  function loadInitialCanvasData(inputData: ArrayBufferLike) {
    let setCount = 0;
    const imgData = ctx.createImageData(1000, 1000);
    let byteInputData = new Uint8Array(inputData);
    const data = imgData.data;
    let bit_index = 0;
    let byte_index = 0;
    for (let i = 0; i < byteInputData.length * 8; i++) {
      bit_index = i % 8;
      byte_index = Math.floor(i / 8);
      let byte_value = byteInputData[byte_index];
      let is_set = is_bit_set(byte_value, bit_index);
      let color;
      if (is_set) {
        //console.log("Found set", byte_index, bit_index);
        setCount += 1;
        color = 0;
        data[i * 4] = 255; //r
        data[i * 4 + 1] = 0; //g
        data[i * 4 + 2] = 0; //b
      } else {
        color = 255;
        data[i * 4] = 255; //r
        data[i * 4 + 1] = 255; //g
        data[i * 4 + 2] = 255; //b
      }
      data[i * 4 + 3] = 255; //a
    }
    //console.log(bit_index);
    ctx.putImageData(imgData, 0, 0);

    console.log(`${setCount} pixels set`);
  }

  function is_bit_set(byte_value: number, bit_index: number) {
    let mask = 1 << bit_index;
    return (byte_value & mask) != 0;
  }

  function ws() {
    socket = new WebSocket("/ws");
    console.log("Connecting");

    socket.onmessage = (event) => {
      //console.log(`[message] Данные получены с сервера: ${event.data}`);
      let data = event.data;
      let color;
      data = JSON.parse(data);
      data.on.forEach((index: number) => {
        color = "#ff0000";
        let y = Math.floor(index / 1000);
        let x = index % 1000;
        setPixel(x, y, color);
      });
      data.off.forEach((index: number) => {
        color = "#ffffff";
        let y = Math.floor(index / 1000);
        let x = index % 1000;
        setPixel(x, y, color);
      });
    };

    socket.onopen = (e) => {
      console.log("[open] Соединение установлено");
      loadCanvas();
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

  function initPanzoom(node: HTMLElement) {
    instance = panzoom(node, {
      smoothScroll: false,
      bounds: true,
      boundsPadding: 0.05,
      maxZoom: 10,
      minZoom: 1,
    });
  }

  function handleCanvasClick(event: MouseEvent) {
    const rect = canvas.getBoundingClientRect();
    const { x: panX, y: panY, scale } = instance.getTransform();

    let canvasX = event.offsetX;
    let canvasY = event.offsetY;

    let x = canvasX;
    let y = canvasY;
    let index = (y * 1000 + x).toString();
    //console.log(index);
    axios.post("/set/" + index).then((d) => {
      let color;
      if (d.data == "1") {
        color = "#000000";
      } else {
        color = "#00ff00";
      }

      setPixel(x, y, color);
    });
  }
</script>

<div id="canvasWrapper">
  <canvas
    use:initPanzoom
    bind:this={canvas}
    id="myCanvas"
    width="1000"
    height="1000"
    on:mouseup={handleCanvasClick}
  ></canvas>
</div>

<style>
  #myCanvas {
    image-rendering: pixelated;
    border: 1px solid #000000;
    box-sizing: border-box;
  }
</style>
