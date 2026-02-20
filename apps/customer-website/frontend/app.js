const mockSyncSummary = {
  lastSuccessfulSyncUtc: "2026-02-20T16:42:00Z",
  bytesBackedUp: 78676992102,
  filesBackedUp: 121453,
};

function formatBytes(bytes) {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let unitIndex = 0;

  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }

  return `${value.toFixed(unitIndex > 1 ? 2 : 0)} ${units[unitIndex]}`;
}

const lastSyncEl = document.getElementById("last-sync");
const dataEl = document.getElementById("data-backed-up");
const fileCountEl = document.getElementById("file-count");

lastSyncEl.textContent = new Date(mockSyncSummary.lastSuccessfulSyncUtc).toLocaleString();
dataEl.textContent = formatBytes(mockSyncSummary.bytesBackedUp);
fileCountEl.textContent = mockSyncSummary.filesBackedUp.toLocaleString();
