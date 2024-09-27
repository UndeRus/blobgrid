<script lang="ts">
  import axios from "axios";
  import { onDestroy, onMount } from "svelte";
  import { base64ToArrayBuffer, loadInitialCanvasData } from "./bit_utils";
  import Panzoom, { type PanzoomObject } from "@panzoom/panzoom";
  import kmeans from "kmeans-ts";

  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D;
  let socket: WebSocket;
  let instance: PanzoomObject;

  let watchPixels = false;

  onMount(() => {
    ctx = canvas.getContext("2d")!!;
    ws();
  });

  function zoomToNextPoint(clusterCenters: Array<[number, number]>) {
    let i = 0;
    function iter() {
      if (!watchPixels) {
        return;
      }
      while (i < clusterCenters.length) {
        const element = clusterCenters[i];
        let x = element[0];
        let y = element[1];
        panAndZoomToPoint(x, y);
        i++;
        setTimeout(iter, 1000);
        console.log("iter", i, x, y);
        return;
      }
    }
    iter();
  }

  function panAndZoomToPoint(x: number, y: number) {
    const scale = 7;
    let xArg = -x + 500;
    let yArg = -y + 500;
    instance.zoom(scale, { animate: true, relative: false });
    instance.pan(xArg, yArg, { animate: true, relative: false });
  }

  onDestroy(() => {});

  function calculateClusters(
    points: Array<[number, number]>
  ): Array<[number, number]> {
    let clustersCount = Math.min(points.length, 3);
    if (clustersCount < 3) {
      return points;
    }
    var output = kmeans(points, 3, "kmeans");
    return output.centroids;
  }

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
    axios.get("/api/grid").then(function (response) {
      // handle success
      let data = base64ToArrayBuffer(response.data);
      loadInitialCanvasData(data, ctx);
    });
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
      let color: string;
      let preColor: string;
      data = JSON.parse(data);
      let onDots = data.on.map((index: number) => indexToXY(index));
      let offDots = data.off.map((index: number) => indexToXY(index));

      onDots.forEach((coords: [number, number]) => {
        color = "#ff0000";
        preColor = "#ff8000"
        let [x, y] = coords;
        setPixel(x, y, preColor);

        setTimeout(() => setPixel(x, y, color), 1000);
      });
      offDots.forEach((coords: [number, number]) => {
        preColor = "#0000ff"
        color = "#ffffff";
        let [x, y] = coords;
        setPixel(x, y, preColor);
        setTimeout(() => setPixel(x, y, color), 1000);
      });

      if (watchPixels) {
        let allDots = onDots.concat(offDots);
        let clusters = calculateClusters(allDots);
        zoomToNextPoint(clusters);
      }
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
    instance = Panzoom(node, {
      canvas: true,
      minScale: 1,
      maxScale: 10,
    });
    node.addEventListener("wheel", instance.zoomWithWheel);
  }

  function indexToXY(index: number): [number, number] {
    let y = Math.floor(index / 1000);
    let x = index % 1000;
    return [x, y];
  }
</script>

<div id="goto">
  <a href="/subgrid">Edit here</a>
  <input id="watch" type="checkbox" bind:value={watchPixels} />
  <label for="watch">Watch</label>
</div>

<div id="canvasWrapper">
  <canvas
    use:initPanzoom
    bind:this={canvas}
    id="myCanvas"
    width="1000"
    height="1000"
  ></canvas>
</div>

<style>
  #myCanvas {
    image-rendering: pixelated;
    border: 1px solid #000000;
    box-sizing: border-box;
  }

  #goto {
    position: absolute;
    z-index: 11111;
    background: rgb(255 255 255 / 70%);
    padding: 1em;
  }

  #canvasWrapper {
    width: 1002px;
    height: 1002px;
  }
</style>
