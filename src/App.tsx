import { RouterProvider } from "react-router-dom";
import { router } from "./routes";

function App() {
  return (
    <RouterProvider
      router={router}
      fallbackElement={
        <div className="flex h-screen items-center justify-center text-sm text-muted-foreground">
          正在加载界面...
        </div>
      }
    />
  );
}

export default App;
