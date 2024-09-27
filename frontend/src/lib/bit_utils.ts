export function loadInitialCanvasData(
  inputData: ArrayBufferLike,
  ctx: CanvasRenderingContext2D
) {
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

export function is_bit_set(byte_value: number, bit_index: number) {
  let mask = 1 << bit_index;
  return (byte_value & mask) != 0;
}

export function base64ToArrayBuffer(base64: string) {
  var binaryString = atob(base64);
  var bytes = new Uint8Array(binaryString.length);
  for (var i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  return bytes.buffer;
}
