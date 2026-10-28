import { mount } from 'svelte';
import App from './App.svelte';
import '@fontsource-variable/bricolage-grotesque';
import '@fontsource-variable/nunito-sans';
import './style.css';

mount(App, { target: document.getElementById('app')! });
