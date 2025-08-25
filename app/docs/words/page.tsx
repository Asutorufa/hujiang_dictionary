"use client"

import { changePriority, countWord, incrementRemindCount, ListWordResponse, queryWord, SaveWordModal } from "@/app/components";
import { Button, Card, CardBody, CardFooter, CardHeader, Modal, ModalBody, ModalContent, ModalFooter, Pagination, Select, SelectItem, Spinner, Tab, Tabs } from "@heroui/react";
import { FC, useEffect, useState } from "react";
import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";


// const ChevronRightIcon = (props: SVGProps<SVGSVGElement>) => {
//     return (
//         <svg
//             aria-hidden="true"
//             fill="none"
//             focusable="false"
//             height="1em"
//             role="presentation"
//             stroke="currentColor"
//             strokeLinecap="round"
//             strokeLinejoin="round"
//             strokeWidth="1.5"
//             viewBox="0 0 24 24"
//             width="1em"
//             {...props}
//         >
//             <path d="m9 18 6-6-6-6" />
//         </svg>
//     );
// };


const PlusIcon = ({ size = 24, width, height, ...props }: { size?: number, width?: number, height?: number }) => {
    return (
        <svg
            aria-hidden="true"
            fill="none"
            focusable="false"
            height={size || height}
            role="presentation"
            viewBox="0 0 24 24"
            width={size || width}
            {...props}
        >
            <g
                fill="none"
                stroke="currentColor"
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={1.5}
            >
                <path d="M6 12h12" />
                <path d="M12 18V6" />
            </g>
        </svg>
    );
};

// const ItemCounter = ({ number }: { number: number }) => (
//     <div className="flex items-center gap-1 text-default-400">
//         <span className="text-small">{number}</span>
//         <ChevronRightIcon className="text-xl" />
//     </div>
// );



export default function Words() {
    const [words, setWords] = useState<ListWordResponse[]>([{
        word: "",
        explain: "",
        add_time: 0,
        update_time: 0,
        reminder_time: 0,
        anki_count: 0,
        priority: 0,
    }]);
    const [page, setPage] = useState<number>(1);
    const [total, setTotal] = useState<number>(1);
    const [loading, setLoading] = useState(false);
    const [open, setOpen] = useState(false);
    const [newWord, setNewWord] = useState<ListWordResponse>({
        word: "",
        explain: "",
        add_time: 0,
        update_time: 0,
        reminder_time: 0,
        anki_count: 0,
        priority: 0,
    });
    const [priority, setPriority] = useState({ index: 0, value: 0, open: false });
    const [orderBy, setOrderBy] = useState("word");


    useEffect(() => {
        countWord((size) => {
            if (size) {
                setTotal(Math.ceil(size / 10))
            }
        })
    }, [setTotal])

    useEffect(() => {
        setLoading(true)
        queryWord(page, 10, orderBy, (data) => {
            if (data) {
                setWords(data)
            }

            setLoading(false)
        })
    }, [page, orderBy])

    // if (loading) {
    //     return <>
    //         <div className="flex items-center justify-center h-screen">
    //             <Spinner />
    //         </div>
    //     </>
    // }

    return <>
        <SaveWordModal open={open} onChange={(p) => setOpen(p)} word={newWord.word} explain={newWord.explain} />
        <ConfirmModal
            title={`Are you sure want to change ${words[priority.index].word}'s priority to ${getPriorityText(priority.value)}?`}
            open={priority.open}
            onChange={(p) => setPriority(prev => ({ ...prev, open: p }))}
            onConfirm={async () => {
                setWords(prev => {
                    const newWords = [...prev]
                    newWords[priority.index].priority = priority.value
                    return newWords
                })
            }}
        />

        <div className="h-15" />

        <div className="p-2">
            {loading &&
                <div className="fixed top-1/2 left-1/2 z-50">
                    <Spinner />
                </div>
            }
            <Select
                variant="bordered"
                className="fixed z-50 top-5 w-25"
                classNames={{
                    base: "backdrop-blur-sm"
                }}
                selectedKeys={[orderBy]}
                onChange={(e) => setOrderBy(e.target.value)}
            >
                <SelectItem key="word">Word</SelectItem>
                <SelectItem key="word desc">Word DESC</SelectItem>
                <SelectItem key="priority">Priority</SelectItem>
                <SelectItem key="priority desc">Priority DESC</SelectItem>
                <SelectItem key="update_time">Time</SelectItem>
                <SelectItem key="update_time desc">Time DESC</SelectItem>
            </Select>

            <div className="flex fixed z-50 top-5 left-1/2 -translate-x-1/2">
                <Pagination
                    // isCompact
                    showControls
                    // showShadow
                    classNames={{
                        prev: "shadow-md backdrop-blur-sm",
                        next: "shadow-md backdrop-blur-sm",
                        item: "shadow-md backdrop-blur-sm"
                    }}
                    variant="bordered"
                    onChange={(p) => { setPage(p) }}
                    page={page}
                    className="items-center"
                    siblings={0} initialPage={page} total={total}


                />
                <Button isIconOnly onPress={() => setOpen(true)} size="md" variant="bordered" className="ms-2 shadow-md backdrop-blur-sm"><PlusIcon /></Button>
            </div>

            <div className="gap-2 top-5">
                {
                    words.length && words.filter(w => w.word.length > 0).map((w, i) =>
                        <Card
                            key={w.word + i}
                            // endContent={<ItemCounter number={w.reminder_time} />}
                            onClick={() => {
                                setNewWord(w)
                                setOpen(true)
                            }}
                            className="mt-2"
                        >
                            <CardHeader className="flex justify-between">
                                <div className="flex gap-2">
                                    <Tabs
                                        size="sm"
                                        variant="bordered"
                                        color={getPriorityColor(w.priority)}
                                        selectedKey={w.priority.toString()}
                                        onSelectionChange={(e) => {
                                            changePriority(w.word, parseInt(e.toString()), (error) => {
                                                if (!error) {
                                                    setPriority({ index: i, value: parseInt(e.toString()), open: true })
                                                }
                                            })
                                        }}
                                    >
                                        <Tab title="Low" key={0} />
                                        <Tab title="Medium" key={1} />
                                        <Tab title="High" key={2} />
                                    </Tabs>
                                    <div className="flex">
                                        <Button
                                            isIconOnly
                                            variant="bordered"
                                            size="md"
                                            onPress={() => {
                                                incrementRemindCount(w.word, (error) => {
                                                    if (!error) {
                                                        setWords(prev => {
                                                            const newWords = [...prev]
                                                            newWords[i].anki_count = newWords[i].anki_count + 1
                                                            return newWords
                                                        })
                                                    }
                                                })
                                            }}
                                        >
                                            <Up />
                                        </Button>
                                        <span className="flex z-10 flex-wrap relative box-border rounded-full whitespace-nowrap place-content-center origin-center items-center select-none font-regular scale-100 opacity-100 subpixel-antialiased data-[invisible=true]:scale-0 data-[invisible=true]:opacity-0 text-small px-0 transition-transform-opacity !ease-soft-spring !duration-300 border-transparent border-0 bg-default text-default-foreground w-5 h-5 min-w-5 min-h-5 top-[20%] right-[35%] translate-x-1/2 -translate-y-1/2">
                                            {w.anki_count}
                                        </span>
                                    </div>
                                </div>
                                {/* <Chip
                                    className="ms-2"
                                    size="sm"
                                    variant="dot"
                                    color={w.priority === 0 ? "success" : w.priority === 1 ? "warning" : "secondary"}
                                    onClick={(e) => { e.stopPropagation() }}
                                >
                                    {w.priority === 0 ? "Low" : w.priority === 1 ? "Medium" : "High"}
                                </Chip> */}

                                <span>{w.word}</span>
                                <span className="text-default-500">{new Date(w.update_time * 1000).toLocaleString()}</span>
                            </CardHeader>
                            <CardBody>
                                <div className="px-2 py-1 rounded-small bg-default-100 group-data-[hover=true]:bg-default-200">
                                    <span className="text-tiny text-default-600">
                                        <Markdown remarkPlugins={[remarkGfm]}>{w.explain}</Markdown>
                                    </span>
                                </div>
                            </CardBody>

                            <CardFooter className="gap-2">
                                <Button
                                    fullWidth
                                    className="border-small border-white/20 bg-white/10 text-white"
                                    onPress={() => {
                                        setNewWord(w)
                                        setOpen(true)
                                    }}
                                >
                                    Edit
                                </Button>

                                <Button
                                    fullWidth
                                    className="border-small border-white/20 bg-white/10 text-white"
                                >
                                    {getPriorityText(w.priority)}
                                </Button>
                            </CardFooter>

                        </Card>
                    )
                }
            </div>
        </div>
    </>
}


function getPriorityColor(priority: number) {
    if (priority === 0) {
        return "success"
    } else if (priority === 1) {
        return "warning"
    } else {
        return "secondary"
    }
}

function getPriorityText(priority: number) {
    if (priority === 0) {
        return "Low"
    } else if (priority === 1) {
        return "Medium"
    } else {
        return "High"
    }
}

const ConfirmModal: FC<{ title: string, open: boolean, onChange: (open: boolean) => void, onConfirm: () => Promise<void> }> = ({ title, open, onConfirm, onChange }) => {
    const [loading, setLoading] = useState(false)

    return <Modal isOpen={open} onOpenChange={onChange} hideCloseButton>
        <ModalContent>
            <>
                <ModalBody className="text-center">{title}</ModalBody>
                <ModalFooter>
                    <Button onPress={() => onChange(false)}>Close</Button>
                    <Button
                        isLoading={loading} color="primary"
                        onPress={async () => {
                            setLoading(true)
                            await onConfirm()
                            setLoading(false)
                            onChange(false)
                        }}
                    >
                        Ok
                    </Button>
                </ModalFooter>
            </>
        </ModalContent>
    </Modal>
}


function Up() {
    return (
        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.5" d="M12 20V4m0 0l6 6m-6-6l-6 6" /></svg>
    )
}