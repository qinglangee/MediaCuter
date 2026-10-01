import { formatTimeForFfmpeg } from './time.js';

export function generateCommand(inputPath, startMs, endMs, outputPath) {
  return `ffmpeg -y -ss ${formatTimeForFfmpeg(startMs)} -i "${inputPath}" -to ${formatTimeForFfmpeg(Math.max(0, endMs - startMs))} -c copy "${outputPath}"`;
}

export function buildFfmpegArgs(inputPath, startMs, endMs, outputPath) {
  return [
    'ffmpeg', '-y',
    '-ss', formatTimeForFfmpeg(startMs),
    '-i', inputPath,
    '-to', formatTimeForFfmpeg(Math.max(0, endMs - startMs)),
    '-c', 'copy',
    outputPath
  ];
}
