#!/usr/bin/env bash

file_as_string=`cat params.json`
n=`echo "$file_as_string" | cut -d "\"" -f 4 `
t=`echo "$file_as_string" | cut -d "\"" -f 8 `

rm -f bin/bs23/*.store
killall sm_manager bs23_keygen_client bs23_presign_client bs23_sign_client bs23_compile_sig 2> /dev/null

if [[ ! -d ./bin/bs23 ]]
then
    if [[ ! -d ./bin ]]
    then
        mkdir "./bin"
    fi
    mkdir "./bin/bs23"
fi
if [[ ! -d ./bin/message ]]
then
    echo "Testing non-interactive threshold ECDSA signing" > ./bin/message
fi

echo -e "\nSM Manager:"
./target/release/examples/sm_manager &
sleep 2

echo -e "\n##################\n# Key generation #\n##################\n"
for i in $(seq 1 $n)
do
    echo "Key-gen for client $i out of $n"
    ./target/release/examples/bs23_keygen_client http://127.0.0.1:8001 bin/bs23/keys$i.store bin/public_key &
    sleep 3
done
sleep 7

echo -e "\n###############\n# Pre-signing #\n###############\n"
for i in $(seq 1 $n)
do
    echo "Pre-signing for client $i out of $n"
    ./target/release/examples/bs23_presign_client http://127.0.0.1:8001 bin/bs23/keys$i.store bin/bs23/presign$i.store &
    sleep 3
done
sleep 7

echo -e "\n###########\n# Signing #\n###########\n"
for i in $(seq 1 $((t+1)));
do
    echo "Signing locally for client $i out of $((t+1))"
    ./target/release/examples/bs23_sign_client bin/bs23/presign$i.store bin/bs23/localsig$i.store bin/message &
    sleep 3
done

echo -e "\n#######################\n# Compiling Signature #\n#######################\n"
for i in $(seq 1 $((t+1)));
do
    echo "Compiling signature $i out of $((t+1))"
    ./target/release/examples/bs23_compile_sig http://127.0.0.1:8001 bin/bs23/localsig$i.store bin/signature &
    sleep 3
done
sleep 7

killall sm_manager 2> /dev/null