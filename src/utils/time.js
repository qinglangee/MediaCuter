export function formatTime(ms) {
  if (ms < 0) ms = 0;
  ms = Math.round(ms);
  const hours = Math.floor(ms / 3600000);
  const minutes = Math.floor((ms % 3600000) / 60000);
  const seconds = Math.floor((ms % 60000) / 1000);
  const millis = ms % 1000;
  return (
    String(hours).padStart(2, '0') + ':' +
    String(minutes).padStart(2, '0') + ':' +
    String(seconds).padStart(2, '0') + '.' +
    String(millis).padStart(3, '0')
  );
}

export function parseTime(str) {
  const trimmed = str.trim();
  const match = trimmed.match(/^(\d{1,2}):(\d{1,2}):(\d{1,2})\.(\d{1,3})$/);
  if (!match) return null;
  const h = parseInt(match[1], 10);
  const m = parseInt(match[2], 10);
  const s = parseInt(match[3], 10);
  const msStr = match[4];
  const ms = parseInt(msStr.padEnd(3, '0'), 10);
  if (m >= 60 || s >= 60) return null;
  return h * 3600000 + m * 60000 + s * 1000 + ms;
}

export function formatTimeForFfmpeg(ms) {
  if (ms < 0) ms = 0;
  const hours = Math.floor(ms / 3600000);
  const minutes = Math.floor((ms % 3600000) / 60000);
  const seconds = Math.floor((ms % 60000) / 1000);
  const millis = ms % 1000;
  return (
    String(hours).padStart(2, '0') + ':' +
    String(minutes).padStart(2, '0') + ':' +
    String(seconds).padStart(2, '0') + '.' +
    String(millis).padStart(3, '0')
  );
}
