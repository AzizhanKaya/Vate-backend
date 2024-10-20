import { createRoot } from 'react-dom/client'
import { RouterProvider } from 'react-router-dom'
import routes from './routes'
import './assets/css/tailwind.css'
import { Provider } from 'react-redux'
import store from './store'
import init from './wasm/wasm';

async function startApp() {
   await init();
 
   createRoot(document.getElementById('root')).render(
     <Provider store={store}>
       <RouterProvider router={routes} />
     </Provider>
   );
 }
 
 startApp();
