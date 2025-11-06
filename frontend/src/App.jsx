import React from "react";
import { Routes, Route } from "react-router-dom";
import Home from "./pages/Home";
import Login from "./pages/Login";
import Register from "./pages/Register";
import Profile from "./pages/Profile";
import Post from "./pages/Post";
import VideoFeed from "./pages/videofeed";
import ForgotPassword from "./pages/Forgotpassword";
import Resetpassword from "./pages/Resetpassword";
import { GlobalToaster } from "./Globaltoaster"; // ✅ correct import

function App() {
  return (
    <>
      {/* ✅ Toaster should be outside the Routes, rendered globally */}
      <GlobalToaster />

      <Routes>
        <Route path="/" element={<Register />} />
        <Route path="/register" element={<Register />} />
        <Route path="/login" element={<Login />} />
        <Route path="/forgotpassword" element={<ForgotPassword />} />
        <Route path="/reset-password" element={<Resetpassword />} />
        <Route path="/home" element={<Home />} />
        <Route path="/profile/:id" element={<Profile />} />
        <Route path="/post" element={<Post />} />
        <Route path="/getpost" element={<VideoFeed />} />
      </Routes>
    </>
  );
}

export default App;
