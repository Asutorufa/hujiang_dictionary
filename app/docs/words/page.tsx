"use client"

import { changePriority, countWord, EditIcon, FilterIcon, incrementRemindCount, ListWordResponse, queryWord, RefreshIcon, SaveWordModal, TrashIcon } from "@/app/components";
import { Button, Card, CardBody, CardHeader, Dropdown, DropdownItem, DropdownMenu, DropdownTrigger, Pagination, Spinner, Tab, Tabs } from "@heroui/react";
import { useEffect, useState } from "react";
import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { useLocalStorage } from "usehooks-ts";


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
    const [total, setTotal] = useState<number>(100);
    const [loading, setLoading] = useState(false);
    const [open, setOpen] = useState(false);
    const [refresh, setRefresh] = useState(0);
    const [newWord, setNewWord] = useState<ListWordResponse>({
        word: "",
        explain: "",
        add_time: 0,
        update_time: 0,
        reminder_time: 0,
        anki_count: 0,
        priority: 0,
    });
    const [orderBy, setOrderBy] = useLocalStorage("order_by", "word");


    useEffect(() => {
        countWord((size) => {
            if (size) {
                setTotal(Math.ceil(size / 10))
            }
        })
    }, [setTotal, refresh])

    useEffect(() => {
        setLoading(true)
        queryWord(page, 10, orderBy, (data, error) => {
            if (data) {
                setWords(data)
            }

            // if (error) {
            //     for (let i = 0; i < 10; i++) {
            //         setWords(prev => [...prev, {
            //             word: refresh + "tttttttttttttttttttttttest" + i,
            //             explain: error,
            //             add_time: 0,
            //             update_time: 0,
            //             reminder_time: 0,
            //             anki_count: 0,
            //             priority: 0,
            //         }])
            //     }
            // }

            setLoading(false)
        })
    }, [page, orderBy, refresh])

    // if (loading) {
    //     return <>
    //         <div className="flex items-center justify-center h-screen">
    //             <Spinner />
    //         </div>
    //     </>
    // }

    return <>
        <SaveWordModal open={open} onChange={(p) => setOpen(p)} word={newWord.word} explain={newWord.explain} />

        <div className="p-2">
            {loading &&
                <div className="fixed top-1/2 left-1/2 z-50">
                    <Spinner />
                </div>
            }

            <div className="sticky flex flex-wrap gap-1 z-50 left-1/2 top-1 justify-center">
                <Pagination
                    isCompact
                    showControls
                    isDisabled={loading}
                    // showShadow
                    classNames={{
                        item: "border-0",
                        wrapper: "shadow-md backdrop-blur-sm border-medium border-default",
                        cursor: "border-medium bg-transparent border-blue-300",
                    }}
                    color="default"
                    variant="bordered"
                    onChange={(p) => { setPage(p) }}
                    page={page}
                    className="items-center"
                    siblings={0}
                    initialPage={page}
                    total={total}
                />

                <div className="flex flex-wrap gap-1">
                    <Dropdown>
                        <DropdownTrigger>
                            <Button isDisabled={loading} isIconOnly size="md" variant="bordered" className="shadow-md backdrop-blur-sm"><FilterIcon /></Button>
                        </DropdownTrigger>
                        <DropdownMenu
                            selectionMode="single"
                            selectedKeys={[orderBy]}
                            onSelectionChange={(e) => setOrderBy(e.currentKey ? e.currentKey : "word")}
                        >
                            <DropdownItem key="word">Word</DropdownItem>
                            <DropdownItem key="word desc">Word DESC</DropdownItem>
                            <DropdownItem key="priority">Priority</DropdownItem>
                            <DropdownItem key="priority desc">Priority DESC</DropdownItem>
                            <DropdownItem key="update_time">Time</DropdownItem>
                            <DropdownItem key="update_time desc">Time DESC</DropdownItem>
                        </DropdownMenu>
                    </Dropdown>

                    <Button isIconOnly onPress={() => setOpen(true)} size="md" variant="bordered" className="shadow-md backdrop-blur-sm">
                        <PlusIcon />
                    </Button>

                    <Button isIconOnly
                        onPress={() => setRefresh(p => p + 1)}
                        size="md" variant="bordered" className="shadow-md backdrop-blur-sm"
                        isDisabled={loading}
                    >
                        <RefreshIcon />
                    </Button>
                </div>
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
                            <CardHeader className="flex justify-between flex-wrap">
                                <span>{w.word}</span>

                                <div className="flex gap-2">
                                    <Tabs
                                        size="sm"
                                        variant="bordered"
                                        color={getPriorityColor(w.priority)}
                                        selectedKey={w.priority.toString()}
                                        onSelectionChange={(e) => {
                                            changePriority(w.word, parseInt(e.toString()), (error) => {
                                                if (!error) {
                                                    setWords(prev => {
                                                        const newWords = [...prev]
                                                        newWords[i].priority = parseInt(e.toString())
                                                        return newWords
                                                    })
                                                }
                                            })
                                        }}
                                    >
                                        <Tab title="Low" key={0} />
                                        <Tab title="Medium" key={1} />
                                        <Tab title="High" key={2} />
                                    </Tabs>
                                </div>
                            </CardHeader>
                            <CardBody>
                                <div className="px-2 py-1 rounded-small bg-default-100 group-data-[hover=true]:bg-default-200">
                                    <span className="text-tiny text-default-600">
                                        <Markdown remarkPlugins={[remarkGfm]}>{w.explain}</Markdown>
                                    </span>
                                </div>

                                <div className="flex  mt-1">

                                    <span className="text-default-500">{new Date(w.update_time * 1000).toLocaleString()}</span>

                                    <div className="ml-auto flex">
                                        <div className="flex">
                                            <Button
                                                isIconOnly
                                                className="bg-transparent"
                                                size="sm"
                                                radius="lg"
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
                                                {w.anki_count}
                                            </Button>
                                        </div>

                                        <Button
                                            isIconOnly
                                            size="sm"
                                            radius="lg"
                                            className="bg-transparent"
                                            onPress={() => {
                                                setNewWord(w)
                                                setOpen(true)
                                            }}
                                        >
                                            <EditIcon />
                                        </Button>

                                        <Button
                                            isIconOnly
                                            size="sm"
                                            radius="lg"
                                            className="bg-transparent"
                                        >
                                            <TrashIcon />
                                        </Button>
                                    </div>
                                </div>
                            </CardBody>
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

function Up() {
    return (
        <svg xmlns="http://www.w3.org/2000/svg" width="17" height="17" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.5" d="M12 20V4m0 0l6 6m-6-6l-6 6" /></svg>
    )
}