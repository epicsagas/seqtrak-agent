import { mount } from 'svelte'
import '@fontsource/noto-sans-kr/400.css'
import '@fontsource/noto-sans-kr/600.css'
import './tokens.css'
import App from './App.svelte'

const app = mount(App, { target: document.getElementById('app')! })

export default app
