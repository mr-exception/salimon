import os from 'node:os';

function toMegabytes(bytes: number) {
  return Math.round((bytes / 1024 / 1024) * 100) / 100;
}

export class MetricsService {
  static getMemoryUsage() {
    const processMemory = process.memoryUsage();
    const systemTotal = os.totalmem();
    const systemFree = os.freemem();
    const systemUsed = systemTotal - systemFree;

    return {
      process: {
        bytes: processMemory,
        megabytes: {
          rss: toMegabytes(processMemory.rss),
          heapTotal: toMegabytes(processMemory.heapTotal),
          heapUsed: toMegabytes(processMemory.heapUsed),
          external: toMegabytes(processMemory.external),
          arrayBuffers: toMegabytes(processMemory.arrayBuffers),
        },
      },
      system: {
        bytes: {
          total: systemTotal,
          free: systemFree,
          used: systemUsed,
        },
        megabytes: {
          total: toMegabytes(systemTotal),
          free: toMegabytes(systemFree),
          used: toMegabytes(systemUsed),
        },
      },
      uptimeSeconds: Math.round(process.uptime()),
      timestamp: new Date().toISOString(),
    };
  }
}
