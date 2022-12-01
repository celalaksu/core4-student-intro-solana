import { Card } from './Card'
import { FC, useEffect, useState } from 'react'
import { StudentIntro } from '../models/StudentIntro'
import * as web3 from '@solana/web3.js'
import { Button, Heading, Center, HStack, Input, Spacer } from '@chakra-ui/react'
import { StudentIntroCoordinator } from '../coordinators/StudentIntroCoordinator'
import { useDisclosure } from "@chakra-ui/react"
import { ReviewDetail } from "./ReviewDetail"
import { useConnection } from "@solana/wallet-adapter-react"

export const StudentIntroList: FC = () => {
    // const connection = new web3.Connection(web3.clusterApiUrl('devnet'))
    const { connection } = useConnection()
    const [studentIntros, setStudentIntros] = useState<StudentIntro[]>([])
    const [page, setPage] = useState(1)
    const [search, setSearch] = useState('')
    const { isOpen, onOpen, onClose } = useDisclosure()
    const [selectedMovie, setSelectedMovie] = useState<StudentIntro>(StudentIntro.mocks[0])


    useEffect(() => {
        StudentIntroCoordinator.fetchPage(
            connection,
            page,
            5,
            search,
            search !== ''
        ).then(setStudentIntros)
    }, [page, search, connection])

    const handleReviewSelected = (studentIntro: StudentIntro) => {
        setSelectedMovie(studentIntro)
        onOpen()
    }
    
    return (
        <div>
            <Center>
                <Input
                    id='search'
                    color='gray.400'
                    onChange={event => setSearch(event.currentTarget.value)}
                    placeholder='Search'
                    w='97%'
                    mt={2}
                    mb={2}
                />
            </Center>
            <Heading as="h1" size="l" color="white" ml={4} mt={8}>
                Select Review To Comment
            </Heading>
            <ReviewDetail
                isOpen={isOpen}
                onClose={onClose}
                studentIntro={selectedMovie ?? studentIntros[0]}
            />
            {
                studentIntros.map((studentIntro, i) => <Card 
                            key={i} 
                            studentIntro={studentIntro}
                            onClick={() => {
                                handleReviewSelected(studentIntro)
                            }}
                            />)
            }
            <Center>
                <HStack w='full' mt={2} mb={8} ml={4} mr={4}>
                    {
                        page > 1 && <Button onClick={() => setPage(page - 1)}>Previous</Button>
                    }
                    <Spacer />
                    {
                        StudentIntroCoordinator.accounts.length > page * 5 &&
                        <Button onClick={() => setPage(page + 1)}>Next</Button>
                    }
                </HStack>
            </Center>
        </div>
    )
}