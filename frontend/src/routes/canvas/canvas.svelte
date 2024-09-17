<script lang="ts">
	let canvas
</script>

<style>
    #myCanvas {
      image-rendering: pixelated;
      border: 1px solid #000000;
    }
</style>

<canvas
bind:this={canvas} 
id="myCanvas"
width="1000"
height="1000"></canvas>
