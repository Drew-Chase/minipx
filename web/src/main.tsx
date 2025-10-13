import React from "react";
import {BrowserRouter, Route, Routes, useNavigate} from "react-router-dom";
import ReactDOM from "react-dom/client";

import "./css/index.css";
import Navigation from "./components/Navigation";
import {ThemeProvider} from "./providers/ThemeProvider";
import {HeroUIProvider} from "@heroui/react";
import Dashboard from "./pages/Dashboard";


ReactDOM.createRoot(document.getElementById("root")!).render(
    <React.StrictMode>
        <BrowserRouter>
            <ThemeProvider>
                <MainContentRenderer/>
            </ThemeProvider>
        </BrowserRouter>
    </React.StrictMode>
);

export function MainContentRenderer()
{
    const navigate = useNavigate();
    return (
        <HeroUIProvider navigate={navigate}>
            <div className="min-h-screen bg-background">
                <Navigation/>
                <Routes>
                    <Route path="/" element={<Dashboard/>}/>
                </Routes>
            </div>
        </HeroUIProvider>
    );
}
