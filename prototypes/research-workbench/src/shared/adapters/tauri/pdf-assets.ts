import workerUrl from 'pdfjs-dist/build/pdf.worker.mjs?url';
const localDirectory=(name:string)=>new URL(`/pdfjs/${name}/`,window.location.href).toString();
export const pdfWorkerUrl=workerUrl;
export const pdfResourceUrls={
 cMapUrl:localDirectory('cmaps'),
 standardFontDataUrl:localDirectory('standard_fonts'),
 iccUrl:localDirectory('iccs'),
 wasmUrl:localDirectory('wasm'),
 useWasm:true,
 cMapPacked:true,
} as const;
