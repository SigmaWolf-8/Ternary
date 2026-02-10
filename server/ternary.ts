export function ternaryEncode(binaryData: Buffer): Buffer {
  const trits: number[] = [];
  for (let i = 0; i < binaryData.length; i++) {
    let byte = binaryData[i];
    for (let j = 0; j < 6; j++) {
      trits.push(byte % 3);
      byte = Math.floor(byte / 3);
    }
  }
  
  const packed: number[] = [];
  for (let i = 0; i < trits.length; i += 4) {
    let value = 0;
    for (let j = Math.min(3, trits.length - i - 1); j >= 0; j--) {
      value = value * 3 + (trits[i + j] || 0);
    }
    packed.push(value);
  }
  return Buffer.from(packed);
}

export function ternaryDecode(ternaryData: Buffer): Buffer {
  const trits: number[] = [];
  for (let i = 0; i < ternaryData.length; i++) {
    let value = ternaryData[i];
    for (let j = 0; j < 4; j++) {
      trits.push(value % 3);
      value = Math.floor(value / 3);
    }
  }
  
  const bytes: number[] = [];
  for (let i = 0; i < trits.length; i += 6) {
    if (i + 5 >= trits.length) break;
    let byte = 0;
    let multiplier = 1;
    for (let j = 0; j < 6; j++) {
      byte += trits[i + j] * multiplier;
      multiplier *= 3;
    }
    if (byte <= 255) {
      bytes.push(byte);
    }
  }
  return Buffer.from(bytes);
}

export function runLengthCompress(data: Buffer): Buffer {
  if (data.length === 0) return Buffer.alloc(0);
  
  const result: number[] = [];
  let i = 0;
  
  while (i < data.length) {
    let count = 1;
    while (i + count < data.length && data[i] === data[i + count] && count < 127) {
      count++;
    }
    
    if (count >= 3) {
      result.push(0x80 | count);
      result.push(data[i]);
      i += count;
    } else {
      let literalStart = i;
      let literalCount = 0;
      
      while (i < data.length && literalCount < 127) {
        let runLength = 1;
        while (i + runLength < data.length && data[i] === data[i + runLength]) {
          runLength++;
        }
        if (runLength >= 3) break;
        i++;
        literalCount++;
      }
      
      if (literalCount > 0) {
        result.push(literalCount);
        for (let j = 0; j < literalCount; j++) {
          result.push(data[literalStart + j]);
        }
      }
    }
  }
  
  return Buffer.from(result);
}

export function runLengthDecompress(data: Buffer): Buffer {
  const result: number[] = [];
  let i = 0;
  
  while (i < data.length) {
    const header = data[i++];
    
    if (header & 0x80) {
      const count = header & 0x7F;
      const value = data[i++];
      for (let j = 0; j < count; j++) {
        result.push(value);
      }
    } else {
      const count = header;
      for (let j = 0; j < count && i < data.length; j++) {
        result.push(data[i++]);
      }
    }
  }
  
  return Buffer.from(result);
}

export interface CompressionResult {
  originalData: string;
  compressedData: string;
  originalSize: number;
  ternarySize: number;
  compressedSize: number;
  compressionRatio: number;
}

export function compressData(jsonData: string): CompressionResult {
  const originalBuffer = Buffer.from(jsonData, 'utf-8');
  const originalSize = originalBuffer.length;
  
  const ternaryEncoded = ternaryEncode(originalBuffer);
  const ternarySize = ternaryEncoded.length;
  
  const rleCompressed = runLengthCompress(ternaryEncoded);
  const compressedSize = rleCompressed.length;
  
  const compressionRatio = ((originalSize - compressedSize) / originalSize) * 100;
  
  const lengthPrefix = Buffer.alloc(4);
  lengthPrefix.writeUInt32BE(originalSize, 0);
  const withPrefix = Buffer.concat([lengthPrefix, rleCompressed]);
  
  return {
    originalData: jsonData,
    compressedData: withPrefix.toString('base64'),
    originalSize,
    ternarySize,
    compressedSize,
    compressionRatio
  };
}

export function decompressData(base64Data: string): string {
  const compressed = Buffer.from(base64Data, 'base64');
  
  const originalSize = compressed.readUInt32BE(0);
  const rleData = compressed.subarray(4);
  
  const ternaryEncoded = runLengthDecompress(rleData);
  const decoded = ternaryDecode(ternaryEncoded);
  
  return decoded.subarray(0, originalSize).toString('utf-8');
}

export function generateSensorData(count: number): object[] {
  const data = [];
  const baseTime = Date.now();
  
  for (let i = 0; i < count; i++) {
    data.push({
      id: i + 1,
      temp: parseFloat((22.5 + Math.sin(i * 0.1) * 5).toFixed(2)),
      humidity: parseFloat((45 + Math.cos(i * 0.1) * 10).toFixed(2)),
      pressure: parseFloat((1013 + Math.sin(i * 0.05) * 20).toFixed(2)),
      timestamp: new Date(baseTime - i * 60000).toISOString()
    });
  }
  return data;
}

export function generateUserEvents(count: number): object[] {
  const events = ["click", "scroll", "hover", "submit", "load"];
  const pages = ["/home", "/products", "/cart", "/checkout", "/profile"];
  const data = [];
  const baseTime = Date.now();
  
  for (let i = 0; i < count; i++) {
    data.push({
      id: i + 1,
      event: events[i % events.length],
      page: pages[i % pages.length],
      userId: `user_${1000 + (i % 50)}`,
      sessionId: `sess_${2000 + Math.floor(i / 10)}`,
      timestamp: new Date(baseTime - i * 30000).toISOString()
    });
  }
  return data;
}

export function generateLogEntries(count: number): object[] {
  const levels = ["INFO", "DEBUG", "WARN", "ERROR", "INFO"];
  const services = ["api", "worker", "scheduler", "gateway", "cache"];
  const data = [];
  const baseTime = Date.now();
  
  for (let i = 0; i < count; i++) {
    data.push({
      id: i + 1,
      level: levels[i % levels.length],
      message: `Process ${i % 10} completed task ${Math.floor(i / 10)}`,
      service: services[i % services.length],
      traceId: `trace_${3000 + i}`,
      timestamp: new Date(baseTime - i * 10000).toISOString()
    });
  }
  return data;
}
