"use client"

import { Button, Listbox, ListboxItem, Modal, ModalBody, ModalContent, ModalFooter, ModalHeader, Pagination, Spinner, Textarea, useDisclosure } from "@heroui/react";
import { FC, SVGProps, useEffect, useState } from "react";
import Markdown from "react-markdown";


export const ChevronRightIcon = (props: SVGProps<SVGSVGElement>) => {
    return (
        <svg
            aria-hidden="true"
            fill="none"
            focusable="false"
            height="1em"
            role="presentation"
            stroke="currentColor"
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth="1.5"
            viewBox="0 0 24 24"
            width="1em"
            {...props}
        >
            <path d="m9 18 6-6-6-6" />
        </svg>
    );
};


export const PlusIcon = ({ size = 24, width, height, ...props }: { size?: number, width?: number, height?: number }) => {
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

export const ItemCounter = ({ number }: { number: number }) => (
    <div className="flex items-center gap-1 text-default-400">
        <span className="text-small">{number}</span>
        <ChevronRightIcon className="text-xl" />
    </div>
);


type ListWordResponse = {
    word: string,
    explain: string,
    add_time: number,
    update_time: number,
    reminder_time: number,
}

async function queryWord(page: number, size: number, callback: (data?: ListWordResponse[], error?: string) => void) {
    fetch("/word/list", {
        method: "POST",
        headers: {},
        body: JSON.stringify({
            page_size: size,
            page_number: page,
        }),
    })
        .then((res) => res.json() as Promise<ListWordResponse[]>)
        .then((data) => {
            callback(data, undefined);
        })
        .catch((error) => {
            callback(undefined, error.message);
        });
}


type CountWordResponse = {
    size: number,
}

async function countWord(callback: (size?: number, error?: string) => void) {
    fetch("/word/count", {
        method: "POST",
        headers: {},
    })
        .then((res) => res.json() as Promise<CountWordResponse>)
        .then((data) => {
            callback(data.size, undefined);
        })
        .catch((error) => {
            callback(undefined, error.message);
        });
}

export async function saveWord(word: string, explain: string, callback: (error?: string) => void) {
    fetch("/word/save", {
        method: "POST",
        headers: {},
        body: JSON.stringify({
            word: word,
            explain: explain,
        }),
    })
        .then(() => {
            callback(undefined);
        })
        .catch((error) => {
            callback(error.message);
        });
}

export const SaveWordModal: FC<{
    open: boolean,
    onChange: (open: boolean) => void,
    word?: string,
    explain?: string,
}> = ({ open, onChange, word, explain }) => {
    const [saving, setSaving] = useState(false)
    const { isOpen, onOpen, onOpenChange } = useDisclosure();

    useEffect(() => {
        if (open) {
            onOpen()
        }
    }, [open, onOpen])

    const [newWord, setNewWord] = useState(word || "");
    const [newExplain, setNewExplain] = useState(explain || "");

    useEffect(() => {
        setNewWord(word || "");
        setNewExplain(explain || "");
    }, [word, explain])

    return <Modal isOpen={isOpen} backdrop="blur" placement="top-center" onOpenChange={(p) => {
        onChange(p)
        onOpenChange()
    }}>
        <ModalContent>
            {(onClose) => (
                <>
                    <ModalHeader className="flex flex-col gap-1">Save Word</ModalHeader>
                    <ModalBody>
                        <Textarea
                            label="Word"
                            isInvalid={newWord.length === 0}
                            errorMessage={"Word is empty"}
                            value={newWord}
                            onChange={(p) => setNewWord(p.target.value)}
                            placeholder="Enter Word"
                            variant="bordered"
                        />
                        <Textarea
                            label="Explain"
                            isInvalid={newExplain.length === 0}
                            errorMessage={"Explain is empty"}
                            value={newExplain}
                            onChange={(p) => setNewExplain(p.target.value)}
                            placeholder="Enter explain"
                            variant="bordered"
                        />
                    </ModalBody>
                    <ModalFooter>
                        <Button color="danger" variant="flat" onPress={onClose}>
                            Close
                        </Button>
                        <Button color="primary" isLoading={saving} onPress={() => {
                            if (newWord.length === 0 || newExplain.length === 0) return
                            setSaving(true)
                            saveWord(newWord, newExplain, () => {
                                setSaving(false)
                                onClose()
                            })
                        }}>
                            Save
                        </Button>
                    </ModalFooter>
                </>
            )}
        </ModalContent>
    </Modal>
}

export default function Words() {
    const [words, setWords] = useState<ListWordResponse[]>([]);
    const [page, setPage] = useState<number>(1);
    const [total, setTotal] = useState<number>(1);
    const [loading, setLoading] = useState(true);
    const [open, setOpen] = useState(false);


    useEffect(() => {
        countWord((size) => {
            if (size) {
                setTotal(Math.ceil(size / 10))
            }
        })
    }, [setTotal])

    useEffect(() => {
        setLoading(true)
        queryWord(page, 10, (data) => {
            if (data) {
                setWords(data)
            }

            setLoading(false)
        })
    }, [page, setWords])

    // if (loading) {
    //     return <>
    //         <div className="flex items-center justify-center h-screen">
    //             <Spinner />
    //         </div>
    //     </>
    // }

    return <>
        <SaveWordModal open={open} onChange={(p) => setOpen(p)} />

        <div className="p-2">
            {loading &&
                <div className="fixed top-1/2 left-1/2 z-50">
                    <Spinner />
                </div>
            }
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
                <Button isIconOnly onPress={() => setOpen(true)} size="md" variant="bordered" className="items-center ms-2 shadow-md backdrop-blur-sm"><PlusIcon /></Button>

            </div>

            <Listbox>
                {
                    words.map((w) =>
                        <ListboxItem
                            key={w.word}
                        // endContent={<ItemCounter number={w.reminder_time} />}
                        >
                            <div className="flex flex-col gap-1">
                                <div className="flex justify-between">
                                    <span>{w.word}</span>
                                    <span className="text-default-500">{new Date(w.update_time * 1000).toLocaleString()}</span>
                                </div>
                                <div className="px-2 py-1 rounded-small bg-default-100 group-data-[hover=true]:bg-default-200">
                                    <span className="text-tiny text-default-600">
                                        <Markdown>{w.explain}</Markdown>
                                    </span>
                                </div>
                            </div>
                        </ListboxItem>
                    )
                }
            </Listbox>
        </div>
    </>
}