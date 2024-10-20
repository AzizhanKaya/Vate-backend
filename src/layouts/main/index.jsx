import { Outlet } from "react-router-dom";
import Sidebar from "./sidebar";
import Rightbar from "./rightbar";
import Modal from "../../pages/modal";
import { useState } from "react";

export default function MainLayout() {
    const [isModal, unMountModal] = useState(false);

    return (
        <div className="w-[1265px] mx-auto flex">
            <Sidebar />
            <main className="flex-1 flex">
                <main className="flex-1 max-w-[700px] border-x border-[#2f3336]">
                    <Outlet />
                </main>
                <div>
                    <Rightbar />
                </div>
            </main>
            {!isModal && (
            <div className="z-30">
                <Modal unMountModal={unMountModal} />
            </div>
            )}
        </div>
    );
}
