/* eslint-disable no-console */
import fs from 'fs';

const binary = fs.readFileSync('./projects/searcher/target/wasm32-unknown-unknown/debug/searcher.wasm');
const rawData = fs.readFileSync('./projects/json-test/data.json');

let logger = (_ptr: number, _length: number) => {};

WebAssembly.instantiate(binary.buffer, { console: { log: (ptr: number, length: number) => logger(ptr, length) } })
  .then(({ instance }) => {
    interface Program {
      memory: WebAssembly.Memory;
      alloc(capacity: number): number;
      dealloc(ptr: number, capacity: number): void;
      convert(ptr: number): number;
      accumulate(ptr: number, min: number, max: number): number;
      BYTE_LENGTH_HISTOGRAM: number;
      BYTE_LENGTH_RECOMMENDATION: number;
    }
    const program = (instance.exports as unknown) as Program;
    const { memory, alloc, dealloc, convert, accumulate } = program;
    logger = (ptr, length) => {
      console.log(Buffer.from(memory.buffer, ptr, length).toString('utf-8'));
    };
    

    console.log(instance.exports);
    const data = writeBuffer(rawData);
    const resultPointer = convert(data.pointer);
    data.dealloc();

    const byteLength = {
      histogram: new Uint32Array(memory.buffer, program.BYTE_LENGTH_HISTOGRAM, 1)[0],
      recommendation: new Uint32Array(memory.buffer, program.BYTE_LENGTH_RECOMMENDATION, 1)[0],
    };
    const size = {
      histogram: byteLength.histogram / Uint32Array.BYTES_PER_ELEMENT,
      recommendation: byteLength.recommendation / Uint32Array.BYTES_PER_ELEMENT,
    };

    const result = new Uint32Array(memory.buffer, resultPointer, size.histogram);
    const resultBase64 = Buffer.from(memory.buffer, resultPointer, byteLength.histogram).toString('base64');
    // dealloc(resultPointer, byteLength.histogram);

    // const r64 = Buffer.from(memory.buffer, resultPointer, byteLength.histogram).toString('base64');
    // const b = Buffer.from(r64, 'base64');
    // console.log(result, r64, new Uint32Array(b.buffer, b.byteOffset, size.histogram));

    const [, , , rollups_pointer, rollups_capacity] = (result as unknown) as [
      start_date: number,
      end_date: number,
      rollup_type: 0 | 1,
      rollups_pointer: number,
      rollups_capacity: number,
      rollups_length: number,
    ];
    console.log(resultPointer, result);

    const recommendations = new Uint32Array(memory.buffer, rollups_pointer, rollups_capacity * size.recommendation);
    const recommendationsBase64 = Buffer.from(
      memory.buffer,
      rollups_pointer,
      rollups_capacity * byteLength.recommendation,
    ).toString('base64');

    // console.log(result, resultBase64);
    // console.log(recommendations, recommendationsBase64);

    {
      const recoMem = writeBuffer(Buffer.from(recommendationsBase64, 'base64'));

      const input = Buffer.from(resultBase64, 'base64');
      input.writeUInt32LE(recoMem.pointer, 3 * Uint32Array.BYTES_PER_ELEMENT);
      const inputMem = writeBuffer(input);
      console.log(new Uint32Array(memory.buffer, accumulate(inputMem.pointer, 0, 1448927000), 2));
      console.log(new Uint32Array(memory.buffer, accumulate(inputMem.pointer, 0, 15089270000), 2));
      recoMem.dealloc();
      inputMem.dealloc();
    }

    function writeBuffer(buffer: Buffer) {
      const pointer = alloc(buffer.byteLength + 1);
      const view = new Uint8Array(memory.buffer, pointer, buffer.byteLength);
      buffer.copy(view);
      return { pointer, dealloc: () => dealloc(pointer, buffer.length) };
    }
  })
  .catch(console.log);
