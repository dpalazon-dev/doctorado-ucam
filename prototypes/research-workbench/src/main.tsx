import {StrictMode} from 'react';
import {createRoot} from 'react-dom/client';
import {App} from './app/App';
import './app/tokens.css';
import {pdfWorkerUrl} from './shared/adapters/tauri/pdf-assets';
void pdfWorkerUrl;
createRoot(document.getElementById('root')!).render(<StrictMode><App/></StrictMode>);
