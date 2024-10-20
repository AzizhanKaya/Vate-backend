import {Popover, PopoverButton, PopoverPanel, Transition} from "@headlessui/react"
import {useAccount} from "@/store/auth/hooks"

export default function Account(){

	const account = useAccount()

    return(
        
		<div className="mt-auto">
			<Popover className="relative">
				{account.pub_key && account.username && (
				<PopoverButton
					className="my-3 p-3 rounded-full hover:bg-[#eff3f41a] w-full flex text-left items-center transition-colors outline-none"
				>
					<img src={account.profile_pic} className="w-12 flex-shrink-0 h-12 rounded-full object-cover" alt=""/>
					<div className="mx-3">
						<h6 className="font-bold leading-[1.25rem] text-[#dcdedf]">{account.username}</h6>
						<div className="text-[#585858] overflow-hidden whitespace-nowrap text-ellipsis max-w-[150px] pl-[1px]">
							@{account.pub_key}
						</div>
					</div>
					<svg viewBox="0 0 24 24" className="ml-auto h-[1.172rem]">
							<path
								fill="#fff"
								d="M3 12c0-1.1.9-2 2-2s2 .9 2 2-.9 2-2 2-2-.9-2-2zm9 2c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm7 0c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2z"
							/>
					</svg>
				</PopoverButton>)
				}
				<Transition
					enter="transition duration-200 ease-out"
					enterFrom="transform opacity-0"
					enterTo="transform opacity-100"
					leave="transition duration-200 ease-out"
					leaveFrom="transform opacity-100"
					leaveTo="transform opacity-0"
				> 
					<PopoverPanel
						className="absolute top-[80px] -translate-y-2 gap-[100px] py-3 w-[320px] z-[1] left-1/2 -translate-x-1/2 bg-[#0b0c0cf8] shadow-box rounded-2xl p-2">

							<div className="flex flex-col">

								<div className="gap-3 rounded-full transition-colors inline-flex p-3 items-center hover:bg-[#eff3f41a] cursor-pointer">
									
									<svg viewBox="0 0 24 24" className="h-7 block"><path fill="#fff" d="M10.54 1.75h2.92l1.57 2.36c.11.17.32.25.53.21l2.53-.59 2.17 2.17-.58 2.54c-.05.2.04.41.21.53l2.36 1.57v2.92l-2.36 1.57c-.17.12-.26.33-.21.53l.58 2.54-2.17 2.17-2.53-.59c-.21-.04-.42.04-.53.21l-1.57 2.36h-2.92l-1.58-2.36c-.11-.17-.32-.25-.52-.21l-2.54.59-2.17-2.17.58-2.54c.05-.2-.03-.41-.21-.53l-2.35-1.57v-2.92L4.1 8.97c.18-.12.26-.33.21-.53L3.73 5.9 5.9 3.73l2.54.59c.2.04.41-.04.52-.21l1.58-2.36zm1.07 2l-.98 1.47C10.05 6.08 9 6.5 7.99 6.27l-1.46-.34-.6.6.33 1.46c.24 1.01-.18 2.07-1.05 2.64l-1.46.98v.78l1.46.98c.87.57 1.29 1.63 1.05 2.64l-.33 1.46.6.6 1.46-.34c1.01-.23 2.06.19 2.64 1.05l.98 1.47h.78l.97-1.47c.58-.86 1.63-1.28 2.65-1.05l1.45.34.61-.6-.34-1.46c-.23-1.01.18-2.07 1.05-2.64l1.47-.98v-.78l-1.47-.98c-.87-.57-1.28-1.63-1.05-2.64l.34-1.46-.61-.6-1.45.34c-1.02.23-2.07-.19-2.65-1.05l-.97-1.47h-.78zM12 10.5c-.83 0-1.5.67-1.5 1.5s.67 1.5 1.5 1.5c.82 0 1.5-.67 1.5-1.5s-.68-1.5-1.5-1.5zM8.5 12c0-1.93 1.56-3.5 3.5-3.5 1.93 0 3.5 1.57 3.5 3.5s-1.57 3.5-3.5 3.5c-1.94 0-3.5-1.57-3.5-3.5z"></path></svg>
									<span className="font-bold content-center text-lg"> Settings</span>
									
								</div>
								
								<div className="gap-2.5 rounded-full transition-colors inline-flex p-3 items-center hover:bg-[#eff3f41a] mb-2 cursor-pointer"
								
								>

									<svg width="28px" height="28px" viewBox="0 0 50 50">
										
										<circle cx="25" cy="25" r="20" stroke="red" strokeWidth="5" fill="none" />
										
										
										<line x1="15" y1="25" x2="35" y2="25" stroke="red" strokeWidth="5" />
									</svg>

									<span className="font-bold content-center text-lg"> Log out</span>

								</div>
							</div>
						
					</PopoverPanel>
				</Transition>
			</Popover>
		</div>
	)
} 