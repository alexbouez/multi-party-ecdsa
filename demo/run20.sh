#!/usr/bin/env bash

file_as_string=`cat params.json`
n=`echo "$file_as_string" | cut -d "\"" -f 4 `
t=`echo "$file_as_string" | cut -d "\"" -f 8 `

rm -f bin/gg20/*.store
rm -f bin/gg20/signature bin/gg20/public_key
killall sm_manager gg20_keygen_client gg20_presign_client gg20_sign_client gg20_compile_sig 2> /dev/null

if [[ ! -d ./bin/gg20 ]]
then
    if [[ ! -d ./bin ]]
    then
        mkdir "./bin"
    fi
    mkdir "./bin/gg20"
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
    ./target/release/examples/gg20_keygen_client http://127.0.0.1:8001 bin/gg20/keys$i.store bin/public_key &
    sleep 3
done
sleep 7

echo -e "\n###############\n# Pre-signing #\n###############\n"
for i in $(seq 1 $((t+1)));
do
    echo "Pre-signing for client $i out of $(($t+1))"
    ./target/release/examples/gg20_presign_client http://127.0.0.1:8001 bin/gg20/keys$i.store bin/gg20/presign$i.store &
    sleep 3
done
sleep 7

echo -e "\n###########\n# Signing #\n###########\n"
for i in $(seq 1 $((t+1)));
do
    echo "Signing locally for client $i out of $((t+1))"
    ./target/release/examples/gg20_sign_client bin/gg20/presign$i.store bin/gg20/localsig$i.store bin/message &
    sleep 3
done

echo -e "\n#######################\n# Compiling Signature #\n#######################\n"
for i in $(seq 1 $((t+1)));
do
    echo "Compiling signature $i out of $((t+1))"
    ./target/release/examples/gg20_compile_sig http://127.0.0.1:8001 bin/gg20/localsig$i.store bin/signature &
    sleep 3
done
sleep 10

killall sm_manager 2> /dev/null
