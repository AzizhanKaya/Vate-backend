export default function Vate({ goToPage }) {
    return (
        <div className="flex flex-col items-center relative pt-[100px]" style={{ willChange: 'transform'}}>
            <div className="scale-[1.75] relative">
                <div className="py-3 flex justify-center items-center w-full h-full relative">
                    <div className="w-[3.25rem] h-[3.25rem] rounded-full flex items-center justify-center effect-shine -mr-1"style={{
                
              }}>
                        <svg width="40" height="40" viewBox="13 13 74 74">
                            <polygon points="50,75 75,25 25,25" fill="none" stroke="white" strokeWidth="5" />
                            <circle cx="50" cy="75" r="10" fill="black" stroke="white" strokeWidth="4" />
                            <circle cx="75" cy="25" r="10" fill="black" stroke="white" strokeWidth="4" />
                            <circle cx="25" cy="25" r="10" fill="black" stroke="white" strokeWidth="4" />
                            <circle cx="25" cy="25" r="4" fill="white" stroke="white" strokeWidth="3" />
                            <circle cx="75" cy="25" r="4" fill="white" stroke="white" strokeWidth="3" />
                            <circle cx="50" cy="75" r="4" fill="white" stroke="white" strokeWidth="3" />
                        </svg>
                    </div>
                    <div className="flex items-center justify-center font-[Vate] text-[25px] font-extrabold effect-shine text-[#6ba4d2] mt-3.5">
                        Vate
                    </div>
                </div>
            </div>
            <p className="relative text-[#c5c5c9] effect-shine font-extralight text-[25px] text-lg text-center py-2"
            >
                Engellenemez bir ağ
            </p>

            <div className="relative flex flex-col gap-3 items-center justify-center border-t-2 border-[#2f3336] py-6 w-[250px] mx-auto">
                <button
                    onClick={() => goToPage(4)}
                    className="rounded-sm text-lg bg-[#4570a5] w-[90%] h-[40px] font-[Vate]"
                >
                    Login
                </button>
                <button
                    onClick={() => goToPage(1)}
                    className="rounded-sm text-lg bg-[#575757] w-[90%] h-[40px] font-[Vate]"
                >
                    Register
                </button>
            </div>
        </div>
    );
}
