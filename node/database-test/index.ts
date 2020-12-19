/* eslint-disable no-console */
import fs from 'fs';

// const binary = fs.readFileSync('./projects/database-test/target/wasm32-unknown-unknown/release/database_test.wasm');
const binary = fs.readFileSync('./projects/database-test/target/wasm32-unknown-unknown/debug/database_test.wasm');

// const REF = {

// };

let logger = (_ptr: number, _length: number) => {};
let resolve = (_ptr: number, _length: number) => {};

const externs = {
  ctx: {
    resolve: (ptr: number, length: number) => resolve(ptr, length),
  },
  console: {
    log: (ptr: number, length: number) => logger(ptr, length),
  },
};

interface User {
  id: number;
  name: string;
}

interface GameInput {
  id: number;
  name: string;
  tags: string[] | null;
  releaseDate: number | null;
  recomendations: {}[] | null;
}

WebAssembly.instantiate(binary.buffer, externs)
  .then(async ({ instance }) => {
    interface Program {
      memory: WebAssembly.Memory;
      alloc(capacity: number): number;
      dealloc(ptr: number, capacity: number): void;
      init(ptr: number): void;
      persist(): void;
      updateUsers(ptr: number): void;
      updateGames(ptr: number): void;
      filterGames(ptr0: number, ptr1: number): void;
      getFullJson(): number;
    }
    const program = (instance.exports as unknown) as Program;
    console.log(program.memory.buffer.byteLength);

    program.memory.grow(17000);
    logger = (ptr, length) => {
      console.log(Buffer.from(program.memory.buffer, ptr, length).toString('utf-8'));
    };
    resolve = (ptr, length) => {
      const val = new Uint32Array(program.memory.buffer, ptr, length / 4);
      switch (val[0]) {
        case 1:
          console.log(Buffer.from(program.memory.buffer, val[1], val[2]).toString('utf-8'));
          break;
        case 2:
          switch (val[1]) {
            case 0: {
              console.log(Buffer.from(program.memory.buffer, val[2], val[3]).toString('utf-8'));
              break;
            }
            case 1: {
              console.log(Buffer.from(program.memory.buffer, val[2], val[3]).toString('base64'));
            }
          }
          break;
      }
    };

    const init = (snapshot: string) => {
      const { pointer, dealloc } = writeSlice(Buffer.from(snapshot, 'base64'));
      program.init(pointer);
      dealloc();
    };

    const persist = () => program.persist();

    const updateUsers = (users: User[]) => {
      const { pointer, dealloc } = writeSlice(Buffer.from(JSON.stringify(users)));
      program.updateUsers(pointer);
      dealloc();
    };

    const updateGames = (games: GameInput[]) => {
      const { pointer, dealloc } = writeSlice(Buffer.from(JSON.stringify(games)));
      program.updateGames(pointer);
      dealloc();
    };

    const getFullJson = () => program.getFullJson();

    // AAAAAAAAAAACAAAAAAAAAAAAAAAAAAAAAwAAAAAAAABmb28BAAAAAQAAAAMAAAAAAAAAYmFy
    if (true) {
      init('AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA');
      // updateUsers(['baz', 'quz', 'foo', 'bar'].map((name, id) => ({ id, name })));
      updateGames([
        { id: 0, name: 'hoge', releaseDate: 20000, recomendations: null, tags: ['Hoge', 'Fuga', 'タグ', '恐竜'] },
        { id: 1, name: 'geqgwq', releaseDate: 20000, recomendations: null, tags: ['Hogeaaa', 'Fuga', 'タグ', '恐竜'] },
        { id: 2, name: 'gwqhwq', releaseDate: 20000, recomendations: null, tags: ['Hoge', 'タグううう', '恐竜'] },
        { id: 3, name: 'gwqhe', releaseDate: 20000, recomendations: null, tags: ['タグ', 'ああああ'] },
      ]);
      // const d = writeSlice(Buffer.from(Uint32Array.from([a.pointer]).buffer));
      // persist();
      // const e = writeSlice(Buffer.from(''));
      // const a = createIdQuery(1, 0, [0]);
      // const b = writeSlice(Buffer.from(''));
      // program.filterGames(a.pointer, b.pointer);
      // await Promise.resolve();
      getFullJson();
      // await Promise.resolve();
      // persist();
    } else {
      init(
        'AAAAAAAAAAAEAAAAAAAAAAIAAAACAAAABgAAAAAAAABnd3Fod3EBBAAAAAAAAAAAAAAAAQAAAAQAAAADAAAAASBOAAAAAAAAAAAAAAAEAAAAAAAAAGhvZ2UBBAAAAAAAAAAAAAAAAQAAAAIAAAADAAAAASBOAAAAAQAAAAEAAAAGAAAAAAAAAGdlcWd3cQEEAAAAAAAAAAAAAAABAAAAAgAAAAMAAAABIE4AAAADAAAAAwAAAAUAAAAAAAAAZ3dxaGUBBAAAAAAAAAAAAAAAAQAAAAIAAAAFAAAAASBOAAAABgAAAAAAAAACAAAABgAAAAAAAADjgr/jgrAFAAAADAAAAAAAAADjgYLjgYLjgYLjgYIDAAAABgAAAAAAAADmgZDnq5wBAAAABAAAAAAAAABGdWdhBAAAAA8AAAAAAAAA44K/44Kw44GG44GG44GGAAAAAAQAAAAAAAAASG9nZQ==',
      );
      console.log(getFullJson());
    }

    function alloc(capacity: number) {
      const pointer = program.alloc(capacity);
      const dealloc = () => program.dealloc(pointer, capacity);
      return { pointer, dealloc };
    }

    function writeBuffer(data: Buffer) {
      const mem = alloc(data.byteLength);
      const view = new Uint8Array(program.memory.buffer, mem.pointer, data.byteLength);
      data.copy(view);
      return mem;
    }

    function writeSlice(data: Buffer) {
      const stackSize = 8; // 4 + 4
      const mem = alloc(stackSize + data.byteLength);
      const heapPointer = mem.pointer + 8;
      const stack = new Uint32Array(program.memory.buffer, mem.pointer, stackSize);
      const heap = new Uint8Array(program.memory.buffer, heapPointer, data.byteLength);
      stack[0] = heapPointer;
      stack[1] = data.byteLength;
      data.copy(heap);
      return mem;
    }

    // function readSlice(ptr: number, itemSize: number) {
    //   const stack = new Uint32Array(program.memory.buffer, ptr, 3);
    //   const heap = new Uint8Array(program.memory.buffer, stack[0], stack[2] * itemSize);
    //   return heap;
    // }

    // function readRawPtr(rawPtr: number) {
    //   const [ptr, size] = new Uint32Array(program.memory.buffer, rawPtr, 2);
    //   program.dealloc(rawPtr, 8);
    //   const dealloc = () => program.dealloc(ptr, size);
    //   return { ptr, dealloc };
    // }

    // function readDatabaseResponse(rawPtr: number) {
    //   const { ptr, dealloc } = readRawPtr(rawPtr);
    //   try {
    //     const kind = new Uint32Array(program.memory.buffer, ptr, 1)[0];
    //     switch (kind) {
    //       case 0:
    //         return Buffer.from(readSlice(ptr + 4, 1)).toString('utf-8');
    //       case 1:
    //         return Buffer.from(readSlice(ptr + 4, 1)).toString('base64');
    //       case 2:
    //         return new Error('Database has not been initialized.');
    //       default:
    //         return null;
    //     }
    //   } finally {
    //     dealloc();
    //   }
    // }

    /**
     * @param kind 0: GameId, 1: TagId
     * @param policy 0: Includes, 1: Excludes
     */
    function createIdQuery(kind: 0 | 1, policy: 0 | 1, ids: number[]) {
      const stackSize = 16; // 4 + 4 + 4 + 4
      const listSize = ids.length * 4;
      const mem = alloc(stackSize + listSize);
      const stack = new Uint32Array(program.memory.buffer, mem.pointer, 4);
      stack[0] = kind;
      stack[1] = policy;
      stack[2] = mem.pointer + stackSize;
      stack[3] = ids.length;
      const list = new Uint32Array(program.memory.buffer, mem.pointer + stackSize, ids.length);
      ids.forEach((id, idx) => {
        list[idx] = id;
      });
      // list.set(ids);
      return mem;
    }
  })
  .catch(console.log);
