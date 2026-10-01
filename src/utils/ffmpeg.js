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

const VIDEO_FORMATS = {
  mp4:  { label: 'MP4 (.mp4)',  args: ['-c:v', 'libx264', '-c:a', 'aac'] },
  mkv:  { label: 'MKV (.mkv)',  args: ['-c:v', 'libx264', '-c:a', 'aac'] },
  avi:  { label: 'AVI (.avi)',  args: ['-c:v', 'mpeg4', '-c:a', 'mp3'] },
  mov:  { label: 'MOV (.mov)',  args: ['-c:v', 'libx264', '-c:a', 'aac'] },
  webm: { label: 'WebM (.webm)', args: ['-c:v', 'libvpx-vp9', '-c:a', 'libopus'] },
  flv:  { label: 'FLV (.flv)',  args: ['-c:v', 'libx264', '-c:a', 'aac'] },
  ts:   { label: 'TS (.ts)',    args: ['-c:v', 'libx264', '-c:a', 'aac'] },
};

const AUDIO_FORMATS = {
  mp3:  { label: 'MP3 (.mp3)',   args: ['-c:a', 'libmp3lame', '-q:a', '2'] },
  wav:  { label: 'WAV (.wav)',   args: ['-c:a', 'pcm_s16le'] },
  ogg:  { label: 'OGG (.ogg)',   args: ['-c:a', 'libvorbis', '-q:a', '5'] },
  flac: { label: 'FLAC (.flac)', args: ['-c:a', 'flac'] },
  aac:  { label: 'AAC (.aac)',   args: ['-c:a', 'aac', '-b:a', '192k'] },
  m4a:  { label: 'M4A (.m4a)',   args: ['-c:a', 'aac', '-b:a', '192k'] },
  wma:  { label: 'WMA (.wma)',   args: ['-c:a', 'wmav2', '-b:a', '192k'] },
};

export function getOutputFormats(hasVideo) {
  const formats = hasVideo ? VIDEO_FORMATS : AUDIO_FORMATS;
  return Object.entries(formats).map(([ext, info]) => ({ ext, label: info.label }));
}

export function generateConvertCommand(inputPath, outputExt, outputPath) {
  const formats = VIDEO_FORMATS;
  const allFormats = { ...VIDEO_FORMATS, ...AUDIO_FORMATS };
  const fmt = allFormats[outputExt];
  if (!fmt) return '';
  const codecArgs = fmt.args.join(' ');
  return `ffmpeg -y -i "${inputPath}" ${codecArgs} "${outputPath}"`;
}

export function buildConvertArgs(inputPath, outputExt, outputPath) {
  const allFormats = { ...VIDEO_FORMATS, ...AUDIO_FORMATS };
  const fmt = allFormats[outputExt];
  if (!fmt) return [];
  return ['ffmpeg', '-y', '-i', inputPath, ...fmt.args, outputPath];
}
