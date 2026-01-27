CREATE OR REPLACE VIEW "DBAMV"."SPECIALTY"
(
	"specialty_provider_code",
	"specialty_code",
	"specialty_name",
	"specialty_status",
	"specialty_primary",
	"specialty_created_date",
	"specialty_updated_date",
	"specialty_all_reason",
	"specialty_all_responsible"
) AS
SELECT TO_CHAR(EM.CD_PRESTADOR) AS "specialty_provider_code",
       TO_CHAR(EM.CD_ESPECIALID) AS "specialty_code",
	   E.DS_ESPECIALID AS "specialty_name",
	   E.SN_ATIVO AS "specialty_status",
	   EM.SN_ESPECIAL_PRINCIPAL AS "specialty_primary",
	   NULL AS "specialty_created_date",
	   NULL AS "specialty_updated_date",
	   NULL AS "specialty_all_reason",
	   NULL AS "specialty_all_responsible"
  FROM DBAMV.ESP_MED EM
 INNER JOIN DBAMV.ESPECIALID E ON EM.CD_ESPECIALID = E.CD_ESPECIALID;