import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { deflateSync } from "node:zlib";

const size = 256;
const bytesPerPixel = 4;
const raw = Buffer.alloc((size * bytesPerPixel + 1) * size);

for (let y = 0; y < size; y += 1) {
  const row = y * (size * bytesPerPixel + 1);
  raw[row] = 0;
  for (let x = 0; x < size; x += 1) {
    const offset = row + 1 + x * bytesPerPixel;
    const t = (x + y) / (size * 2);
    raw[offset] = Math.round(22 + 28 * t);
    raw[offset + 1] = Math.round(74 + 92 * t);
    raw[offset + 2] = Math.round(82 + 58 * t);
    raw[offset + 3] = 255;
  }
}

const gold = [245, 196, 107, 255];
const light = [246, 250, 252, 255];
const green = [84, 212, 160, 255];

for (let i = 0; i < 8; i += 1) {
  rect(34 + i, 34 + i, size - 34 - i, 42 + i, gold);
  rect(34 + i, size - 42 - i, size - 34 - i, size - 34 - i, gold);
  rect(34 + i, 34 + i, 42 + i, size - 34 - i, gold);
  rect(size - 42 - i, 34 + i, size - 34 - i, size - 34 - i, gold);
}

rect(72, 74, 94, 182, light);
rect(94, 74, 142, 94, light);
rect(94, 162, 142, 182, light);
rect(94, 114, 130, 134, light);

for (let i = 0; i < 22; i += 1) {
  rect(154 + i, 76 + i, 176 + i, 98 + i, green);
  rect(176 - i, 76 + i, 198 - i, 98 + i, green);
}

const png = Buffer.concat([
  Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
  chunk("IHDR", ihdr(size, size)),
  chunk("IDAT", deflateSync(raw)),
  chunk("IEND", Buffer.alloc(0)),
]);

const output = join("src-tauri", "icons", "icon.png");
mkdirSync(dirname(output), { recursive: true });
writeFileSync(output, png);

const icoOutput = join("src-tauri", "icons", "icon.ico");
writeFileSync(icoOutput, icoFromPng(png));

function rect(x0, y0, x1, y1, color) {
  for (let y = Math.max(0, y0); y < Math.min(size, y1); y += 1) {
    const row = y * (size * bytesPerPixel + 1);
    for (let x = Math.max(0, x0); x < Math.min(size, x1); x += 1) {
      const offset = row + 1 + x * bytesPerPixel;
      raw[offset] = color[0];
      raw[offset + 1] = color[1];
      raw[offset + 2] = color[2];
      raw[offset + 3] = color[3];
    }
  }
}

function ihdr(width, height) {
  const buffer = Buffer.alloc(13);
  buffer.writeUInt32BE(width, 0);
  buffer.writeUInt32BE(height, 4);
  buffer[8] = 8;
  buffer[9] = 6;
  buffer[10] = 0;
  buffer[11] = 0;
  buffer[12] = 0;
  return buffer;
}

function chunk(type, data) {
  const typeBuffer = Buffer.from(type, "ascii");
  const length = Buffer.alloc(4);
  length.writeUInt32BE(data.length, 0);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(Buffer.concat([typeBuffer, data])), 0);
  return Buffer.concat([length, typeBuffer, data, crc]);
}

function icoFromPng(pngBuffer) {
  const headerSize = 6;
  const directorySize = 16;
  const imageOffset = headerSize + directorySize;
  const header = Buffer.alloc(headerSize);
  header.writeUInt16LE(0, 0);
  header.writeUInt16LE(1, 2);
  header.writeUInt16LE(1, 4);

  const directory = Buffer.alloc(directorySize);
  directory[0] = 0;
  directory[1] = 0;
  directory[2] = 0;
  directory[3] = 0;
  directory.writeUInt16LE(1, 4);
  directory.writeUInt16LE(32, 6);
  directory.writeUInt32LE(pngBuffer.length, 8);
  directory.writeUInt32LE(imageOffset, 12);

  return Buffer.concat([header, directory, pngBuffer]);
}

function crc32(buffer) {
  let crc = 0xffffffff;
  for (const byte of buffer) {
    crc ^= byte;
    for (let i = 0; i < 8; i += 1) {
      crc = (crc >>> 1) ^ (0xedb88320 & -(crc & 1));
    }
  }
  return (crc ^ 0xffffffff) >>> 0;
}
