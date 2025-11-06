import { toast, ToastContainer } from "react-toastify";
// Import the main CSS file
import "react-toastify/dist/ReactToastify.css";

export const GlobalToaster = () => (
  <ToastContainer
    position="top-right"
    autoClose={3000} // duration in ms
    hideProgressBar={false}
    newestOnTop={false}
    closeOnClick
    rtl={false}
    pauseOnFocusLoss
    draggable
    pauseOnHover
    style={{
    }}
  />
);

export const toaster = {
  success: (msg) => toast.success(msg, {
   style: { background: "#2e5a2fff", color: "#fff" }, 
  }),
  error: (msg) => toast.error(msg, {
    style: { background: "#bd443cff", color: "#fff" }, 
  }),
  info: (msg) => toast.info(msg), 
  loading: (msg) => toast.loading(msg),
  dismiss: (id) => toast.dismiss(id), 
};
