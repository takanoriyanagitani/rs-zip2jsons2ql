#!/bin/sh

query='query ($basename: String!, $itemname: String!) {

	i0: json(itemName: $itemname, basename: $basename)
	i1: json(itemName: "j1.json", basename: $basename)
	i2: json(itemName: "j2.json", basename: $basename)
	i3: json(itemName: "j3.json", basename: $basename)

}'

a0(){
	jq \
		-c \
		-n \
		--arg q "${query}" \
		--arg base "a0.zip" \
		--arg item "j4.json" \
		'{
	
			query: $q,
			variables: {
				basename: $base,
				itemname: $item,
			}
	}' |
		curl \
			--silent \
			--fail \
			--show-error \
			--data @- \
			http://127.0.0.1:8039 |
		jq .data |
		jq '{
			i0: .i0 | fromjson,
			i1: .i1 | fromjson,
			i2: .i2 | fromjson,
			i3: .i3 | fromjson,
		}' |
		jq '[.i0, .i1, .i2, .i3]' |
		jq -c '.[]'
}

a1(){
	jq \
		-c \
		-n \
		--arg q "${query}" \
		--arg base "a1.zip" \
		--arg item "j2.json" \
		'{
	
			query: $q,
			variables: {
				basename: $base,
				itemname: $item,
			}
	}' |
		curl \
			--silent \
			--fail \
			--show-error \
			--data @- \
			http://127.0.0.1:8039 |
		jq .data |
		jq '{
			i0: .i0 | fromjson,
			i1: .i1 | fromjson,
			i2: .i2 | fromjson,
			i3: .i3 | fromjson,
		}' |
		jq '[.i0, .i1, .i2, .i3]' |
		jq -c '.[]'
}

echo archive 0
a0

echo
echo archive 1
a1
